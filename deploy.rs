#!/usr/bin/env cargo +nightly -Zscript

---
[package]
edition = "2021"
version = "0.1.0"
[dependencies]
clap = { version = "4", features = ["derive"] }
cmd_lib = { version = "1.9.5" }
---

use clap::Parser;
use cmd_lib::*;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, help = "Path to config")]
    config: Option<std::path::PathBuf>,
}

#[cmd_lib::main]
fn main() -> CmdResult {
    let _args = Args::parse();

    run_cmd! (
        warn "You're deploying reflective to production"
    )?;

    let status = run_fun! (
        git status --porcelain
    )?;

    println!("{}", status.len());

    Ok(())
}
