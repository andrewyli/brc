#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::ptr::null;

use clap::Parser;
use memmap2::{Mmap, MmapOptions};
use rapidhash::RapidHashMap;
use std::fs::File;
use std::{fmt, slice};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

#[derive(Default, Clone)]
struct Statistics {
    max: i16,
    min: i16,
    avg: i64,
    count: u32,
}

impl fmt::Display for Statistics {
    // This function must match this signature exactly
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use write! (with a 'w') to send output to the formatter 'f'
        // It works exactly like println!
        write!(
            f,
            "{:.1}/{:.1}/{:.1}",
            self.min as f32 * 0.1,
            self.avg as f32 / self.count as f32 * 0.1,
            self.max as f32 * 0.1
        )
    }
}

#[target_feature(enable = "avx2")]
fn load_delim_masks(ptr: *const u8) -> (u32, u32) {
    let chunk = unsafe { _mm256_loadu_si256(ptr as *const __m256i) };
    (
        (_mm256_movemask_epi8(_mm256_cmpeq_epi8(_mm256_set1_epi8(b';' as i8), chunk)) as u32),
        (_mm256_movemask_epi8(_mm256_cmpeq_epi8(_mm256_set1_epi8(b'\n' as i8), chunk)) as u32),
    )
}

#[target_feature(enable = "avx2")]
fn get_statistics(mmap: &Mmap) -> String {
    let mut temp_statistics: RapidHashMap<String, Statistics> = RapidHashMap::default();
    let mut ptr = mmap.as_ptr();
    let end_ptr = unsafe { ptr.add(mmap.len()) };
    let mut next_is_semi = true;
    let mut record_start = ptr;
    let mut semi_ptr = null();
    while unsafe { ptr.add(32) } < end_ptr {
        let (mut mask_semi, mut mask_nl) = load_delim_masks(ptr);
        loop {
            if next_is_semi {
                if mask_semi == 0 {
                    break;
                }
                let idx = mask_semi.trailing_zeros();
                mask_semi &= mask_semi.wrapping_sub(1);
                semi_ptr = unsafe { ptr.add(idx as usize) };
                let keep_mask = !((1u32 << idx).wrapping_shl(1).wrapping_sub(1));
                mask_nl &= keep_mask;
                next_is_semi = false;
            }
            if !next_is_semi {
                if mask_nl == 0 {
                    break;
                }
                let idx = mask_nl.trailing_zeros();
                mask_nl &= mask_nl.wrapping_sub(1);
                let nl_ptr = unsafe { ptr.add(idx as usize) };

                let name_len = unsafe { semi_ptr.offset_from(record_start) as usize };
                let temp_len = unsafe { nl_ptr.offset_from(semi_ptr.add(1)) as usize };
                let name_slice = unsafe { slice::from_raw_parts(record_start, name_len) };
                let temp_slice = unsafe { slice::from_raw_parts(semi_ptr, temp_len + 1) };
                update_statistics(&mut temp_statistics, name_slice, temp_slice);
                record_start = unsafe { nl_ptr.add(1) };
                next_is_semi = true;
            }
        }
        ptr = unsafe { ptr.add(32) };
    }
    while ptr < end_ptr {
        if next_is_semi && unsafe { ptr.read() } == b';' {
            semi_ptr = ptr;
            next_is_semi = false;
        }
        if !next_is_semi && unsafe { ptr.read() } == b'\n' {
            let idx = ptr as u32;
            let nl_ptr = unsafe { ptr.add(idx as usize) };

            let name_len = unsafe { semi_ptr.offset_from(record_start) as usize };
            let temp_len = unsafe { nl_ptr.offset_from(semi_ptr.add(1)) as usize };
            let name_slice = unsafe { slice::from_raw_parts(record_start, name_len) };
            let temp_slice = unsafe { slice::from_raw_parts(semi_ptr, temp_len + 1) };
            update_statistics(&mut temp_statistics, name_slice, temp_slice);
            record_start = unsafe { nl_ptr.add(1) };
            next_is_semi = true;
        }
        ptr = unsafe { ptr.add(1) };
    }

    let mut keys: Vec<_> = temp_statistics.keys().collect();
    keys.sort();
    let mut out_msg = "".to_owned();
    for key in keys {
        let stat = temp_statistics.get(key).unwrap();
        out_msg = format!("{}{}={},", out_msg, key, stat);
    }
    out_msg[..out_msg.len() - 1].to_string()
}

fn update_statistics(
    stat_map: &mut RapidHashMap<String, Statistics>,
    label_raw: &[u8],
    val_raw: &[u8],
) {
    let city_name: &str = unsafe { str::from_utf8_unchecked(label_raw) };
    let temp: i16 = atoi_times_ten(val_raw);
    if let Some(val) = stat_map.get_mut(city_name) {
        (*val).max = std::cmp::max((*val).max, temp);
        (*val).min = std::cmp::min((*val).min, temp);
        (*val).avg += temp as i64;
        (*val).count += 1;
    } else {
        stat_map.insert(
            city_name.to_owned(),
            Statistics {
                max: temp,
                min: temp,
                avg: temp as i64,
                count: 1,
            },
        );
    }
}

fn main() {
    let args = Args::parse();
    let file = File::open(args.input).unwrap();
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    println!("{{{}}}", unsafe { get_statistics(&mmap) });
}

pub fn atoi_times_ten(s: &[u8]) -> i16 {
    let padding = 1;
    let is_negative = unsafe { *s.get_unchecked(padding) } == b'-';
    let sign: i16 = !is_negative as i16 * 2 - 1;
    let offset = is_negative as usize;
    let is_long = unsafe { *s.get_unchecked(padding + offset + 2) } == b'.';
    let short_offset = !is_long as usize;
    let start = padding - short_offset + offset;
    let val = unsafe { s.get_unchecked(start) }.saturating_sub(b'0') as i16 * 100 * is_long as i16
        + (unsafe { s.get_unchecked(start + 1) } - b'0') as i16 * 10
        + (unsafe { s.get_unchecked(start + 3) } - b'0') as i16;
    val * sign
}

#[cfg(test)]
mod test {

    use crate::atoi_times_ten;

    #[test]
    pub fn atoi_float() {
        let s = ";32.8".as_bytes();
        let i: i16 = atoi_times_ten(s);
        println!("i: {}", i);
        assert_eq!(328, i);
    }
}
