use memmap2::MmapOptions;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

fn main() {
    let args = Args::parse();
    let mut temp_maxes: HashMap<String, f32> = HashMap::new();
    let mut temp_mins: HashMap<String, f32> = HashMap::new();
    let mut temp_avgs: HashMap<String, f32> = HashMap::new();
    let mut temp_counts: HashMap<String, u32> = HashMap::new();
    BufReader::new(File::open(args.input).unwrap())
        .lines()
        .for_each(|line| {
            let line = line.unwrap();
            let mut parts = line.split(';');
            let city_name = parts.next().unwrap();
            let temp: f32 = parts.next().unwrap().parse().unwrap();
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
        });
    let mut keys: Vec<_> = temp_maxes.keys().collect();
    keys.sort();
    let mut out_msg = "{".to_owned();
    for key in keys {
        out_msg = format!(
            "{}{}={:.1}/{:.1}/{:.1},",
            out_msg,
            key,
            temp_mins.get(key).unwrap(),
            temp_avgs.get(key).unwrap() / *temp_counts.get(key).unwrap() as f32,
            temp_maxes.get(key).unwrap()
        );
    }
    println!("{}}}", &out_msg[..out_msg.len() - 1]);
}
