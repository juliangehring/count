use assert_cmd::prelude::*;
use std::error::Error;
use std::process::Command;

#[test]
fn test_toplevel_1k_counts() -> Result<(), Box<Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg(test_file);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_toplevel_1k_counts_top5() -> Result<(), Box<Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--top").arg("5").arg(test_file);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_no_input() -> Result<(), Box<Error>> {
    let test_file = "no_test_file_here";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg(test_file);
    cmd.assert().failure();

    Ok(())
}
