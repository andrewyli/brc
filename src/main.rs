use atoi::atoi;
use clap::Parser;
use itertools::Itertools;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

struct Statistics {
    max: i16,
    min: i16,
    avg: i16,
    count: i16,
}

fn main() {
    let args = Args::parse();
    let mut temp_statistics: HashMap<String, Statistics> = HashMap::new();
    let file = File::open(args.input).unwrap();
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    for (city_name_raw, temp_raw) in mmap.split(|&b| b == b'\n' || b == b';').tuples() {
        let city_name: &str = unsafe { str::from_utf8_unchecked(city_name_raw) };
        let temp: i16 = atoi_times_ten(temp_raw);
        if let Some(val) = temp_statistics.get_mut(city_name) {
            (*val).max = std::cmp::max((*val).max, temp);
            (*val).min = std::cmp::min((*val).min, temp);
            (*val).avg += temp;
            (*val).count += 1;
        } else {
            temp_statistics.insert(
                city_name.to_owned(),
                Statistics {
                    max: temp,
                    min: temp,
                    avg: temp,
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
        out_msg = format!(
            "{}{}={:.1}/{:.1}/{:.1},",
            out_msg,
            key,
            stat.min as f32 * 0.1,
            stat.avg as f32 / stat.count as f32 * 0.1,
            stat.max as f32 * 0.1,
        );
    }
    println!("{}}}", &out_msg[..out_msg.len() - 1]);
}

pub fn atoi_times_ten(s: &[u8]) -> i16 {
    atoi::<i16>(&s[..s.len() - 2]).unwrap_or(0) * 10 + (s[s.len() - 1] - b'0') as i16
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
