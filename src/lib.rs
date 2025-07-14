use data_encoding::HEXLOWER;
use rayon::prelude::*;
use ring::digest;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::io::{Error, Read};
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    Error,
    Fail,
    Ok,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Outcome {
    pub message: String,
    pub status: Status,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            Status::Error => write!(f, "{}: ERROR", self.message),
            Status::Fail => write!(f, "{}: FAILED", self.message),
            Status::Ok => write!(f, "{}: OK", self.message),
        }
    }
}

pub trait HandleResult {
    fn handle_result(&self) -> usize;
}

impl HandleResult for Vec<Outcome> {
    fn handle_result(&self) -> usize {
        let errors_code = handle_errors(self);
        let failures_code = handle_failures(self);
        errors_code + failures_code
    }
}

fn handle_errors(outcomes: &[Outcome]) -> usize {
    // Count errors and return the appropriate status code.
    match outcomes
        .iter()
        .filter(|o| o.status == Status::Error)
        .count()
    {
        count if count > 0 => {
            eprintln!("WARNING: {count} error(s) occured while checking.");
            1
        }
        _ => 0,
    }
}

fn handle_failures(outcomes: &[Outcome]) -> usize {
    // Count failures and return the appropriate status code.
    match outcomes.iter().filter(|o| o.status == Status::Fail).count() {
        count if count > 0 => {
            eprintln!("WARNING: {count} computed checksums did NOT match.");
            1
        }
        _ => 0,
    }
}

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

pub fn verify_files(input: String) -> Vec<Outcome> {
    input
        .par_lines()
        .map(|line| {
            if let Some((file_digest, file_name)) = line.split_once(' ') {
                verify_file(Path::new(file_name.trim()), file_digest.trim())
            } else {
                let outcome = Outcome {
                    message: format!(
                        "Checksum and filename could not be read from line: {line}"
                    ),
                    status: Status::Error,
                };
                eprintln!("{outcome}");
                outcome
            }
        })
        .collect()
}

fn verify_file(file: &Path, file_digest: &str) -> Outcome {
    let outcome = match File::open(file) {
        Ok(fd) => match get_digest(fd) {
            Ok(digest) => Outcome {
                message: format!("{}", file.display()),
                status: if digest == file_digest {
                    Status::Ok
                } else {
                    Status::Fail
                },
            },
            Err(error) => Outcome {
                message: format!("{}: Failed to compute checksum: {error}", file.display()),
                status: Status::Error,
            },
        },
        // File::open failed.
        Err(error) => Outcome {
            message: format!("{}: {error}", file.display()),
            status: Status::Error,
        },
    };
    match outcome.status {
        Status::Ok => println!("{outcome}"),
        _ => eprintln!("{outcome}"),
    };
    outcome
}

pub fn handle_file(file: &Path, bsd_style: bool) -> Outcome {
    match File::open(file) {
        Ok(input) => match get_digest(input) {
            Ok(digest) => {
                let message = if bsd_style {
                    format!("SHA256 ({}) = {}", file.display(), digest)
                } else {
                    format!("{}  {}", digest, file.display())
                };
                Outcome {
                    message,
                    status: Status::Ok,
                }
            }
            Err(error) => Outcome {
                message: format!("{}: Failed to compute checksum: {error}", file.display()),
                status: Status::Error,
            },
        },
        // File::open failed.
        Err(error) => Outcome {
            message: format!("{}: {error}", file.display()),
            status: Status::Error,
        },
    }
}

#[cfg(test)]
mod tests;
