use std::fs::File;
use std::io::Write;
use tempdir::TempDir;

use super::*;

#[test]
fn test_get_digest() {
    // let tmp_dir = TempDir::new("example")?;
    // let file_path = tmp_dir.path().join("my-temporary-note.txt");
    // let mut tmp_file = File::create(file_path)?;
    // writeln!(tmp_file, "Brian was here. Briefly.")?;
    let tmp = TempDir::new("test_sha256sum-rs").unwrap();
    let files = vec![
        [
            "test1.txt",
            "b5c9873a1a82d7878a66bf2ef4aaf1a7c8be0ca64c6a716c4ef80a0c3f1c3e7c",
        ],
        [
            "test2.txt",
            "c4171ad05395669c61b627287c3d4ee21517b4f9e797643285e5b5c268b4e257",
        ],
        [
            "test3.txt",
            "559fdeb3ef127c3deb87af2f373e3a291e59c080f50d6f4c3e48d4e341f12f3b",
        ],
        [
            "test4.txt",
            "78a6c1bef0f634dc6558792a16e9bcecd7a111b69b00986220a670c34755b984",
        ],
    ];
    for [f, expected] in files {
        println!("{f}, expected: {expected}");
        let file_path = tmp.path().join(f);
        let mut temp_file = File::create(&file_path).unwrap();
        writeln!(temp_file, "testing {f:?}").unwrap();

        let input = File::open(file_path).unwrap();
        let digest = get_digest(input).unwrap();
        assert_eq!(digest, expected.to_string());
    }
}
