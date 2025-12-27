use atoi::atoi;
use itertools::Itertools;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

fn main() {
    let args = Args::parse();
    let mut temp_maxes: HashMap<String, i16> = HashMap::new();
    let mut temp_mins: HashMap<String, i16> = HashMap::new();
    let mut temp_avgs: HashMap<String, i16> = HashMap::new();
    let mut temp_counts: HashMap<String, i16> = HashMap::new();
    let file = File::open(args.input).unwrap();
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    #[cfg(target_os = "linux")]
    match mmap.advise(memmap2::Advice::Sequential) {
        Ok(x) => x,
        Err(x) => panic!("{}", x),
    }
    for (city_name_raw, temp_raw) in mmap.split(|&b| b == b'\n' || b == b';').tuples() {
        let city_name: &str = unsafe { str::from_utf8_unchecked(city_name_raw) };
        let temp: i16 = atoi_times_ten(temp_raw);
        if let Some(val) = temp_maxes.get(city_name) {
            if *val < temp {
                temp_maxes.insert(city_name.to_owned(), temp);
            }
        } else {
            temp_maxes.insert(city_name.to_owned(), temp);
        }
        if let Some(val) = temp_mins.get(city_name) {
            if *val > temp {
                temp_mins.insert(city_name.to_owned(), temp);
            }
        } else {
            temp_mins.insert(city_name.to_owned(), temp);
        }
        let count = temp_counts.entry(city_name.to_owned()).or_insert(0);
        if let Some(val) = temp_avgs.get_mut(city_name) {
            *val += temp;
        } else {
            temp_avgs.insert(city_name.to_owned(), temp);
        }
        *count += 1;
    }

    let mut keys: Vec<_> = temp_maxes.keys().collect();
    keys.sort();
    let mut out_msg = "{".to_owned();
    for key in keys {
        out_msg = format!(
            "{}{}={:.1}/{:.1}/{:.1},",
            out_msg,
            key,
            *temp_mins.get(key).unwrap() as f32 * 0.1,
            *temp_avgs.get(key).unwrap() as f32 / *temp_counts.get(key).unwrap() as f32 * 0.1,
            *temp_maxes.get(key).unwrap() as f32 * 0.1,
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
