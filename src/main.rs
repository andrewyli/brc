use atoi::atoi;
use clap::Parser;
use itertools::Itertools;
use memmap2::{Mmap, MmapOptions};
use std::cmp::{max, min};
use std::fs::File;

use std::fmt;
use std::fmt::{Display, Result};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
}

#[derive(Clone, Default, Debug)]
struct Statistics {
    max: i16,
    min: i16,
    avg: i64,
    count: i32,
}

impl Statistics {
    pub fn add_val(&mut self, val: i16) {
        self.max = max(self.max, val);
        self.min = min(self.min, val);
        self.avg += val as i64;
        self.count += 1;
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        write!(
            f,
            "{:.1}/{:.1}/{:.1}",
            self.min as f32 * 0.1,
            self.avg as f32 / self.count as f32 * 0.1,
            self.max as f32 * 0.1,
        )
    }
}

struct StatisticsTrieNode {
    children: [u32; 256],
    stat: Option<Statistics>,
    parent: u32,
    val: u8,
}

struct StatisticsTrie {
    nodes: Vec<StatisticsTrieNode>,
}

impl StatisticsTrie {
    pub fn new() -> Self {
        let mut new_trie = StatisticsTrie {
            nodes: Vec::with_capacity(1 << 16),
        };
        new_trie.nodes.push(StatisticsTrieNode {
            stat: None,
            parent: 0,
            children: [0; 256],
            val: 0,
        });
        new_trie
    }

    #[cfg(test)]
    pub fn get_stat(&self, arr: &[u8]) -> Option<Statistics> {
        let mut cur_idx: u32 = 0;
        for &b in arr {
            let next_idx = self.nodes[cur_idx as usize].children[b as usize];
            if next_idx == 0 {
                return None;
            }
            cur_idx = next_idx;
        }
        self.nodes[cur_idx as usize].stat.clone()
    }

    pub fn insert(&mut self, arr: &[u8], val: i16) {
        let mut cur_idx: u32 = 0;
        for &b in arr {
            cur_idx = self.get_or_create_child(cur_idx, b);
        }
        self.nodes[cur_idx as usize]
            .stat
            .get_or_insert_with(|| Statistics::default())
            .add_val(val);
    }

    pub fn find_child(&self, parent_idx: u32, b: u8) -> u32 {
        unsafe { self.nodes.get_unchecked(parent_idx as usize) }.children[b as usize]
    }

    pub fn get_or_create_child(&mut self, parent_idx: u32, b: u8) -> u32 {
        let child_idx = self.find_child(parent_idx, b);
        if child_idx > 0 {
            return child_idx;
        }
        let new_node = StatisticsTrieNode {
            stat: None,
            parent: parent_idx,
            children: [0; 256],
            val: b,
        };
        let new_idx = self.nodes.len() as u32;
        self.nodes.push(new_node);
        unsafe { self.nodes.get_unchecked_mut(parent_idx as usize) }.children[b as usize] = new_idx;
        new_idx
    }

    pub fn get_all_words_and_stats(&self) -> Vec<(String, &Statistics)> {
        let mut pairs = Vec::new();
        for node_idx in 0..self.nodes.len() {
            if !self.nodes[node_idx].stat.is_none() {
                pairs.push(self.get_word_and_stat(node_idx as u32));
            }
        }
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }

    pub fn get_word_and_stat(&self, leaf_idx: u32) -> (String, &Statistics) {
        let stat = unsafe { self.nodes.get_unchecked(leaf_idx as usize) }
            .stat
            .as_ref()
            .unwrap();
        let mut word = Vec::new();
        let mut cur_idx = leaf_idx;
        while cur_idx != 0 {
            word.push(unsafe { self.nodes.get_unchecked(cur_idx as usize) }.val);
            cur_idx = unsafe { self.nodes.get_unchecked(cur_idx as usize) }.parent;
        }
        word.reverse();
        (unsafe { String::from_utf8_unchecked(word) }, stat)
    }
}

fn main() {
    let args = Args::parse();
    let mut temp_statistics: StatisticsTrie = StatisticsTrie::new();
    let file = File::open(args.input).unwrap();
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    for (city_name_raw, temp_raw) in mmap.split(|&b| b == b'\n' || b == b';').tuples() {
        let temp: i16 = atoi_times_ten(temp_raw);
        temp_statistics.insert(city_name_raw, temp);
    }

    let words_and_stats = temp_statistics.get_all_words_and_stats();
    println!(
        "{}",
        words_and_stats.iter().format_with(", ", |(key, val), f| {
            f(&format_args!("{}={}", key, val))
        })
    );
}

pub fn atoi_times_ten(s: &[u8]) -> i16 {
    atoi::<i16>(&s[..s.len() - 2]).unwrap_or(0) * 10 + (s[s.len() - 1] - b'0') as i16
}

#[cfg(test)]
mod test {

    use crate::{StatisticsTrie, atoi_times_ten};

    #[test]
    pub fn atoi_float() {
        let s = "32.8".as_bytes();
        let i: i16 = atoi_times_ten(s);
        assert_eq!(328, i);
    }

    #[test]
    pub fn test_trie() {
        let word = "abc".as_bytes();
        let mut trie = StatisticsTrie::new();
        trie.insert(word, 67);
        assert!(trie.get_stat(b"a").is_none());
        assert!(trie.get_stat(b"ab").is_none());
        assert!(!trie.get_stat(b"abc").is_none());
        println!("{}", trie.get_stat(b"abc").unwrap());
    }
}
