#![allow(dead_code)]

use std::env::args;

use log::info;

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

    let mut threads = 1;

    if args.len() > 2 {
        threads = args.get(2).unwrap().parse::<usize>().unwrap();
    }

    info!("start download = {}, threads = {}", start, threads);

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    download::download_chems(start);
}
