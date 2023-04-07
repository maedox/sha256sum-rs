use std::{
    fs::read_to_string,
    io::{stdin, Read},
    path::PathBuf,
    process::exit,
};

use rayon::prelude::*;
use sha256sum_rs::{get_digest, handle_file, verify_files, Status};
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();
    let mut result = 0;

    match &args.path.len() {
        // Zero files given as arguments, read from stdin
        0 => {
            let mut input = String::new();
            stdin().read_to_string(&mut input)?;
            if args.check {
                result = verify_files(input)
                    .iter()
                    // keep only the ones that are not ok so we can exit(1) if non-empty.
                    .filter(|o| o.status != Status::Ok)
                    .count();
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
                result = verify_files(content)
                    .iter()
                    // keep only the ones that are not ok so we can exit(1) if non-empty.
                    .filter(|o| o.status != Status::Ok)
                    .count();
            } else {
                // Iterate over all file names in parallel and print digest.
                result = args
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
                    // keep only the ones that are not ok so we can exit(1) if non-empty.
                    .filter(|o| o.status != Status::Ok)
                    .count();
            }
        }
    }
    if result != 0 {
        exit(1)
    }
    Ok(())
}
