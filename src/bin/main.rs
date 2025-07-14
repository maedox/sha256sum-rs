use rayon::prelude::*;
use sha256sum_rs::{get_digest, handle_file, verify_files, HandleResult, Outcome, Status};
use std::{
    fs::read_to_string,
    io::{stdin, Read},
    path::PathBuf,
    process::exit,
};
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

    let result = match &args.path.len() {
        // Zero files given as arguments, read from stdin
        0 => {
            let mut input = String::new();
            stdin().read_to_string(&mut input)?;
            if args.check {
                verify_files(input).handle_result()
            } else {
                let digest = get_digest(input.as_bytes())?;

                if args.bsd_style {
                    println!("SHA256 (-) = {digest}");
                } else {
                    println!("{digest}  -");
                }
                0
            }
        }
        _ => {
            if args.check {
                // Iterate over all file names in parallel and verify checksums.
                args.path
                    .par_iter()
                    .flat_map(|p| {
                        let input = read_to_string(p).unwrap();
                        verify_files(input)
                    })
                    .collect::<Vec<Outcome>>()
                    .handle_result()
            } else {
                // Iterate over all file names in parallel and print digest.
                args.path
                    .par_iter()
                    .map(|p| {
                        let outcome = handle_file(p, args.bsd_style);
                        match outcome.status {
                            Status::Ok => println!("{}", outcome.message),
                            _ => eprintln!("{}", outcome.message),
                        }
                        outcome
                    })
                    .collect::<Vec<Outcome>>()
                    .handle_result()
            }
        }
    };
    if result != 0 {
        exit(result as i32)
    }
    Ok(())
}
