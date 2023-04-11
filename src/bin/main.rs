use std::{
    fs::read_to_string,
    io::{stdin, Read},
    path::PathBuf,
    process::exit,
};

use rayon::prelude::*;
use sha256sum_rs::{get_digest, handle_file, verify_files, Outcome, Status};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "sha256sum-rs",
    about = "Rust implementation of sha256sum from coreutils"
)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    path: Vec<PathBuf>,

    #[structopt(
        short = "-c",
        long = "--check",
        help = "read SHA256 sums from the FILEs and check them"
    )]
    check: bool,

    #[structopt(long = "--tag", help = "create a BSD-style checksum")]
    bsd_style: bool,
}

fn handle_result(outcomes: Vec<Outcome>) -> usize {
    let mut code = 0;
    let count_errors = outcomes
        .iter()
        .filter(|o| o.status == Status::Error)
        .count();
    let count_fails = outcomes.iter().filter(|o| o.status == Status::Fail).count();
    if count_errors > 0 {
        eprintln!(
            "WARNING: {} error(s) occured while verifying checksums.",
            count_errors
        );
        code += 1;
    };
    if count_fails > 0 {
        eprintln!("WARNING: {} computed checksums did NOT match.", count_fails);
        code += 1;
    };
    code
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();
    let mut result = 0;

    match &args.path.len() {
        // Zero files given as arguments, read from stdin
        0 => {
            let mut input = String::new();
            stdin().read_to_string(&mut input)?;
            if args.check {
                let outcomes = verify_files(input);
                result = handle_result(outcomes);
            } else {
                let digest = get_digest(input.as_bytes())?;

                if args.bsd_style {
                    println!("SHA256 (-) = {}", digest);
                } else {
                    println!("{}  -", digest);
                }
            }
        }
        _ => {
            if args.check {
                if args.path.len() > 1 {
                    eprintln!("Only a single file can be supplied when checking.");
                    exit(1);
                }
                let content = read_to_string(&args.path[0]).unwrap();
                let outcomes = verify_files(content);
                result = handle_result(outcomes);
            } else {
                // Iterate over all file names in parallel and print digest.
                let outcomes = args
                    .path
                    .par_iter()
                    .map(|p| {
                        let outcome = handle_file(p, args.bsd_style);
                        match outcome.status {
                            Status::Ok => println!("{}", outcome.message),
                            _ => eprintln!("{}", outcome.message),
                        }
                        outcome
                    })
                    .collect();
                result = handle_result(outcomes);
            }
        }
    }
    if result != 0 {
        exit(result as i32)
    }
    Ok(())
}
