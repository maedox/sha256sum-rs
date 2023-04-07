use std::{
    io::{stdin, Read},
    path::PathBuf,
};

use rayon::prelude::*;
use sha256sum_rs::{get_digest, handle_file};
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
        _ => args
            .path
            .par_iter()
            .for_each(|p| match handle_file(p, args.check, args.bsd_style) {
                Ok(_) => (),
                Err(err) => eprintln!("{err}"),
            }),
    }
    Ok(())
}
