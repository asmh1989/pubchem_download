#![allow(dead_code)]

use std::env::args;

mod config;
mod download;

fn main() {
    // println!("Hello, world!");
    crate::config::init_config();

    let args: Vec<String> = args().collect();
    let mut start: usize = 0;
    if args.len() > 1 {
        start = args.get(1).unwrap().parse::<usize>().unwrap();
    }

    download::download_chems(start);
}
