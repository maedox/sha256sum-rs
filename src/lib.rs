use data_encoding::HEXLOWER;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use ring::digest;
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::io::{Error, Read};
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

pub fn check_files(input: String) -> Result<i32, Error> {
    // TODO: Rip out this collect and iter on par_lines of input instead?
    let lines = input.lines().collect::<Vec<&str>>();

    let passed = lines
        .par_iter()
        .map(|line| {
            let mut result = None;

            if let Some((file_digest, file_name)) = line.split_once("  ") {
                match File::open(file_name) {
                    Ok(input) => match get_digest(input) {
                        Ok(digest) => {
                            if digest == file_digest {
                                println!("{file_name}: OK");
                                result = Some(());
                            } else {
                                println!("{file_name}: FAILED");
                            }
                        }
                        Err(_) => {
                            println!("{file_name}: FAILED");
                        }
                    },
                    Err(error) => eprintln!("{file_name:?}: {error}"),
                }
            } else {
                println!("{line}: FAILED");
            };

            result
        })
        .filter_map(|v| v)
        .count();

    if lines.len() == passed {
        Ok(0)
    } else {
        Ok(1)
    }
}

pub fn handle_file(file: &Path, check: bool, bsd_style: bool) -> Result<(), Error> {
    match File::open(file) {
        Ok(input) => {
            if check {
                let content = read_to_string(file)?;
                match check_files(content) {
                    Ok(exit_code) => std::process::exit(exit_code),
                    Err(error) => eprintln!("{:?}: {}", file, error),
                }
            } else {
                let digest = get_digest(input)?;

                if bsd_style {
                    println!("SHA256 ({}) = {}", file.display(), digest);
                } else {
                    println!("{}  {}", digest, file.display());
                }
            }
        }

        Err(error) => eprintln!("{:?}: {}", &file, error),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
