#!/usr/bin/env cargo +nightly -Zscript

---
[package]
edition = "2021"
[dependencies]
clap = { version = "4", features = ["derive"] }
---

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, help = "Path to config")]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
