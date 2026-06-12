//! End-to-end tests that exercise the compiled `jsonlz4` binary.

use std::io::Write;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("jsonlz4").unwrap()
}

const SAMPLE_JSON: &[u8] =
    br#"{"bookmarks":[{"title":"hello","children":[1,2,3,4,5,6,7,8,9,10]}]}"#;

#[test]
fn compress_then_decompress_round_trip_via_files() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.json");
    let compressed = dir.path().join("out.jsonlz4");
    let decompressed = dir.path().join("out.json");

    std::fs::write(&input, SAMPLE_JSON).unwrap();

    cmd()
        .arg("--compress")
        .arg(&input)
        .arg(&compressed)
        .assert()
        .success();

    let bytes = std::fs::read(&compressed).unwrap();
    assert_eq!(&bytes[..8], jsonlz4::MAGIC, "compressed file missing magic");

    cmd()
        .arg("--decompress")
        .arg(&compressed)
        .arg(&decompressed)
        .assert()
        .success();

    assert_eq!(std::fs::read(&decompressed).unwrap(), SAMPLE_JSON);
}

#[test]
fn decompress_is_default_mode() {
    let dir = tempfile::tempdir().unwrap();
    let compressed = dir.path().join("in.jsonlz4");
    std::fs::write(&compressed, jsonlz4::encode(SAMPLE_JSON).unwrap()).unwrap();

    let output = cmd().arg(&compressed).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, SAMPLE_JSON);
}

#[test]
fn pipe_through_stdin_and_stdout() {
    let mut compress = cmd()
        .arg("-c")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    compress
        .stdin
        .as_mut()
        .unwrap()
        .write_all(SAMPLE_JSON)
        .unwrap();
    let compressed = compress.wait_with_output().unwrap();
    assert!(compressed.status.success());
    assert_eq!(&compressed.stdout[..8], jsonlz4::MAGIC);

    let mut decompress = cmd()
        .arg("-d")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    decompress
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&compressed.stdout)
        .unwrap();
    let out = decompress.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, SAMPLE_JSON);
}

#[test]
fn rejects_non_jsonlz4_input() {
    let dir = tempfile::tempdir().unwrap();
    let bogus = dir.path().join("bogus.bin");
    std::fs::write(&bogus, b"this is not a jsonlz4 file at all").unwrap();

    cmd()
        .arg(&bogus)
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid magic"));
}

#[test]
fn rejects_too_short_input() {
    let dir = tempfile::tempdir().unwrap();
    let tiny = dir.path().join("tiny.bin");
    std::fs::write(&tiny, b"abc").unwrap();

    cmd()
        .arg(&tiny)
        .assert()
        .failure()
        .stderr(predicate::str::contains("too short"));
}

#[test]
fn compress_and_decompress_flags_are_mutually_exclusive() {
    cmd()
        .arg("-c")
        .arg("-d")
        .arg("anything")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used"));
}
