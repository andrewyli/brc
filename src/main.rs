use clap::Parser;
use itertools::Itertools;
use memmap2::{Mmap, MmapOptions};
use rapidhash::RapidHashMap;
use std::fmt;
use std::fs::File;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

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

fn main() {
    let args = Args::parse();
    let mut temp_statistics: RapidHashMap<String, Statistics> = RapidHashMap::default();
    let file = File::open(args.input).unwrap();
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    for (city_name_raw, temp_raw) in mmap.split(|&b| b == b'\n' || b == b';').tuples() {
        let city_name: &str = unsafe { str::from_utf8_unchecked(city_name_raw) };
        let temp: i16 = atoi_times_ten(temp_raw);
        if let Some(val) = temp_statistics.get_mut(city_name) {
            (*val).max = std::cmp::max((*val).max, temp);
            (*val).min = std::cmp::min((*val).min, temp);
            (*val).avg += temp as i64;
            (*val).count += 1;
        } else {
            temp_statistics.insert(
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

    let mut keys: Vec<_> = temp_statistics.keys().collect();
    keys.sort();
    let mut out_msg = "{".to_owned();
    for key in keys {
        let stat = temp_statistics.get(key).unwrap();
        out_msg = format!("{}{}={},", out_msg, key, stat);
    }
    println!("{}}}", &out_msg[..out_msg.len() - 1]);
}

pub fn atoi_times_ten(s: &[u8]) -> i16 {
    let is_negative = s[0] == b'-';
    let offset = is_negative as usize;
    let length = s.len();
    let val = if length - offset == 3 {
        (s[offset] - b'0') as i16 * 10 + (s[offset + 2] - b'0') as i16
    } else {
        (s[offset] - b'0') as i16 * 100
            + (s[offset + 1] - b'0') as i16 * 10
            + (s[offset + 3] - b'0') as i16
    };
    if is_negative { -val } else { val }
}

#[cfg(test)]
mod test {

    use crate::atoi_times_ten;

    #[test]
    pub fn atoi_float() {
        let s = "32.8".as_bytes();
        let i: i16 = atoi_times_ten(s);
        assert_eq!(328, i);
    }
}
