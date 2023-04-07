use data_encoding::HEXLOWER;
use rayon::prelude::*;
use ring::digest;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::io::{Error, Read};
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    Error,
    Fail,
    Ok,
}

#[derive(Debug)]
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

pub fn check_files(input: String) -> Vec<Outcome> {
    input
        .par_lines()
        .map(|line| {
            if let Some((file_digest, file_name)) = line.split_once("  ") {
                check_line(file_name, file_digest)
            } else {
                Outcome {
                    message: line.to_string(),
                    status: Status::Error,
                }
            }
        })
        .collect()
}

fn check_line(file_name: &str, file_digest: &str) -> Outcome {
    match File::open(file_name) {
        Ok(fd) => match get_digest(fd) {
            Ok(digest) => {
                if digest == file_digest {
                    Outcome {
                        message: file_name.to_string(),
                        status: Status::Ok,
                    }
                } else {
                    Outcome {
                        message: file_name.to_string(),
                        status: Status::Fail,
                    }
                }
            }
            Err(err) => Outcome {
                message: err.to_string(),
                status: Status::Error,
            },
        },
        Err(error) => Outcome {
            message: format!("{file_name}: {error}"),
            status: Status::Error,
        },
    }
}

pub fn handle_file(file: &Path, check: bool, bsd_style: bool) -> Vec<Outcome> {
    match File::open(file) {
        Ok(input) => {
            if check {
                // TODO: This won't print progress, only everything when finished.
                let content = read_to_string(file).unwrap();
                check_files(content)
            } else {
                match get_digest(input) {
                    Ok(digest) => {
                        let result = if bsd_style {
                            format!("SHA256 ({}) = {}", file.display(), digest)
                        } else {
                            format!("{}  {}", digest, file.display())
                        };
                        vec![Outcome {
                            message: result,
                            status: Status::Ok,
                        }]
                    }
                    Err(error) => vec![Outcome {
                        message: format!("{}: {error}", file.display()),
                        status: Status::Error,
                    }],
                }
            }
        }
        Err(error) => vec![Outcome {
            message: format!("{}: {error}", file.display()),
            status: Status::Error,
        }],
    }
}

#[cfg(test)]
mod tests;
