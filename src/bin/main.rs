use std::{
    io::{stdin, Read},
    path::PathBuf,
    process::exit,
};

use rayon::prelude::*;
use sha256sum_rs::{get_digest, handle_file, Status};
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

    match &args.path.len() {
        // Zero files given as arguments, read from stdin
        0 => {
            let mut input = String::new();
            stdin().read_to_string(&mut input)?;
            let digest = get_digest(input.as_bytes())?;

            if args.bsd_style {
                println!("SHA256 (-) = {}", digest);
            } else {
                println!("{}  -", digest);
            }
        }
        // Iterate over all file names in parallel and print digest.
        _ => {
            let result = args
                .path
                .par_iter()
                // handle_file returns Vec<Outcome> so flatten/explode that Vec.
                .flat_map(|p| handle_file(p, args.check, args.bsd_style))
                // map over values so we can print to stdout.
                .map(|o| {
                    if args.check {
                        println!("{o}");
                    } else {
                        println!("{}", o.message);
                    }
                    o
                })
                // keep only the ones that are not ok so we can exit(1) if non-empty.
                .filter(|o| o.status != Status::Ok)
                .count();
            if result != 0 {
                exit(1)
            }
        }
    }
    Ok(())
}
