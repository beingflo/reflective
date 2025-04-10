#!/usr/bin/env cargo +nightly -Zscript

---
[package]
edition = "2021"
version = "0.1.0"
[dependencies]
clap = { version = "4", features = ["derive"] }
cmd_lib = { version = "1.9.5" }
semver = "1.0.26"
---

use clap::Parser;
use cmd_lib::*;
use semver::Version;
use std::io;
use std::io::Write;
use std::io::Read;
use std::fs;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, help = "Path to config")]
    config: Option<std::path::PathBuf>,
}

#[cmd_lib::main]
fn main() -> CmdResult {
    let _args = Args::parse();

    run_cmd! (warn "You're deploying reflective to production")?;

    let status = run_fun! (git status --porcelain)?;

    if status.len() > 0 {
        run_cmd! (error "There are uncommitted changes")?;
        //return Ok(());
    }

    let version = run_fun! (
        cd service;
        cargo metadata --format-version=1 --no-deps | jq ".packages[0].version"
    )?;

    let version = version.trim().replace('"', "");

    let version = Version::parse(&version).unwrap();

    let new_version = {
        let mut version = version.clone();
        version.patch = version.patch + 1;
        version
    };

    println!("Current version: {}", version);

    print!("Enter new version (default {}): ", new_version);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let new_version = if input.trim().is_empty() {
        new_version
    } else {
        match Version::parse(input.trim()) {
            Ok(v) => v,
            Err(_) => {
                run_cmd! (error "Invalid version" $input)?;
                return Ok(());
            }
        }
    };

    println!("Releasing version {}", new_version);

    run_cmd! (
        cd service;
        cargo set-version "$new_version";
        cd ..;
        docker buildx build --platform=linux/amd64 -t "ghcr.io/beingflo/reflective:$new_version" .;
        docker push "ghcr.io/beingflo/reflective:$new_version";
    )?;

    let contents = fs::read_to_string("./docker-compose.prod.yml").expect("Couldn't read docker compose file");
    let contents = contents.replace(&format!("ghcr.io/beingflo/reflective:{}", version), &format!("ghcr.io/beingflo/reflective:{}", new_version));
    fs::write("./docker-compose.prod.yml", contents).expect("Couldn't write docker compose file");

    run_cmd! (
        docker --context omni compose --file docker-compose.prod.yml pull;
        docker --context omni compose --file docker-compose.prod.yml up -d;
        git commit -am "Release $new_version";
        git tag "$new_version";
        git push origin --tags;
    )?;


    Ok(())
}
