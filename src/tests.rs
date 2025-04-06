use std::fs::File;
use std::io::{Error, Write};
use std::sync::LazyLock;
use tempfile::tempdir;

use super::*;

struct TestFile {
    file: &'static str,
    expected_digest: &'static str,
}

/// Some test files, which will match the given digest if their contents are
/// "testing {file_name:?}"
static FILES: LazyLock<&[TestFile]> = LazyLock::new(|| {
    &[
        TestFile {
            file: "test1.txt",
            expected_digest: "b5c9873a1a82d7878a66bf2ef4aaf1a7c8be0ca64c6a716c4ef80a0c3f1c3e7c",
        },
        TestFile {
            file: "test2.txt",
            expected_digest: "c4171ad05395669c61b627287c3d4ee21517b4f9e797643285e5b5c268b4e257",
        },
        TestFile {
            file: "test3.txt",
            expected_digest: "559fdeb3ef127c3deb87af2f373e3a291e59c080f50d6f4c3e48d4e341f12f3b",
        },
        TestFile {
            file: "test4.txt",
            expected_digest: "78a6c1bef0f634dc6558792a16e9bcecd7a111b69b00986220a670c34755b984",
        },
    ]
});

#[test]
fn test_get_digest() -> Result<(), Error> {
    // Directly test the get_digest fn returns the expected digest.

    let tmp = tempdir()?;

    for TestFile {
        file,
        expected_digest,
    } in FILES.iter()
    {
        let file_path = tmp.path().join(file);
        let mut temp_file = File::create(&file_path)?;
        writeln!(temp_file, "testing {file:?}")?;

        let input = File::open(file_path)?;
        let digest = get_digest(input)?;
        assert_eq!(digest, expected_digest.to_string());
    }
    Ok(())
}

#[test]
fn test_handle_file_open_error() {
    // Test that status is Error if a nonexistent file is given to handle_file.

    let result = handle_file(Path::new("nonexisting_file"), false);
    assert_eq!(result.status, Status::Error)
}

#[test]
fn test_handle_file() -> Result<(), Error> {
    // Hash the content of test files and verify the digest matches the expected string.

    let tmp = tempdir()?;
    for TestFile {
        file,
        expected_digest,
    } in FILES.iter()
    {
        let file_path = tmp.path().join(file);
        let mut temp_file = File::create(&file_path)?;
        writeln!(temp_file, "testing {file:?}")?;

        let outcome = handle_file(file_path.as_path(), false);
        let expected_outcome = Outcome {
            message: format!("{expected_digest}  {}", file_path.display()),
            status: Status::Ok,
        };
        assert_eq!(outcome, expected_outcome);
    }
    Ok(())
}

#[test]
fn test_verify_file() -> Result<(), Error> {
    // Write a line to the test files and test that verify_file status is Ok.

    let tmp = tempdir()?;
    for TestFile {
        file,
        expected_digest,
    } in FILES.iter()
    {
        let file_path = tmp.path().join(file);
        let mut temp_file = File::create(&file_path)?;
        writeln!(temp_file, "testing {file:?}")?;

        let outcome = verify_file(file_path.as_path(), expected_digest);
        assert_eq!(outcome.status, Status::Ok)
    }
    Ok(())
}

#[test]
fn test_verify_file_fails() -> Result<(), Error> {
    // Write the wrong content to the test files and make sure status is Fail.

    let tmp = tempdir()?;

    for TestFile {
        file,
        expected_digest,
    } in FILES.iter()
    {
        let file_path = tmp.path().join(file);
        let mut temp_file = File::create(&file_path)?;
        writeln!(temp_file, "fails {file:?}")?;

        let outcome = verify_file(file_path.as_path(), expected_digest);
        assert_eq!(outcome.status, Status::Fail)
    }
    Ok(())
}

#[test]
fn test_verify_files() -> Result<(), Error> {
    // Read a String containing a digest and filename per line, then verify them.

    let tmp = tempdir()?;
    let mut check_string = String::new();

    for TestFile {
        file,
        expected_digest,
    } in FILES.iter()
    {
        let file_path = tmp.path().join(file);
        let mut temp_file = File::create(&file_path)?;
        writeln!(temp_file, "testing {file:?}")?;
        check_string += format!("{expected_digest}  {}\n", file_path.display()).as_ref();
    }
    for outcome in verify_files(check_string) {
        assert_eq!(outcome.status, Status::Ok)
    }
    Ok(())
}
