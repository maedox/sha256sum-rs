use data_encoding::HEXLOWER;
use rayon::prelude::*;
use ring::digest;
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::io::{Error, Read};
use std::ops::Deref;
use std::path::Path;

fn sha256_digest<R: Read>(mut reader: R) -> Result<digest::Digest, Error> {
    let mut context = digest::Context::new(&digest::SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn get_digest<R: Read>(input: R) -> Result<String, Error> {
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader)?;
    Ok(HEXLOWER.encode(digest.as_ref()))
}

pub fn check_files<T: Deref<Target = str> + ToString + Send + Sync>(input: T) {
    // TODO: Rip out this collect and iter on par_lines of input instead?
    let lines: Vec<&str> = input.lines().collect();

    lines.par_iter().for_each(|line| {
        if let Some((file_digest, file_name)) = line.split_once("  ") {
            match check_line(file_name, file_digest) {
                Ok(text) => println!("{text}"),
                Err(error) => println!("{error}"),
            }
        } else {
            println!("{line}: FAILED")
        }
    });
}

fn check_line(file_name: &str, file_digest: &str) -> Result<String, Error> {
    let fd = File::open(file_name)?;
    match get_digest(fd) {
        Ok(digest) => {
            if digest == file_digest {
                Ok(format!("{file_name}: OK"))
            } else {
                Ok(format!("{file_name}: FAILED"))
            }
        }
        Err(err) => Err(err),
    }
}

pub fn handle_file(file: &Path, check: bool, bsd_style: bool) -> Result<(), Error> {
    if file.is_file() {
        match File::open(file) {
            Ok(input) => {
                if check {
                    let content = read_to_string(file)?;
                    check_files(content);
                } else {
                    let digest = get_digest(input)?;

                    if bsd_style {
                        println!("SHA256 ({}) = {}", file.display(), digest);
                    } else {
                        println!("{}  {}", digest, file.display());
                    }
                }
            }
            Err(error) => {
                eprintln!("{:?}: {}", &file, error);
            }
        }
    } else {
        eprintln!("{}: Is not a file.", file.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
