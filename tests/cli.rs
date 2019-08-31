use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, ends_with, starts_with},
};
use std::{
    process::Command,
    error::Error
};

#[test]
fn test_toplevel_1k_counts() -> Result<(), Box<dyn Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg(test_file);
    cmd.assert()
        .success()
        .stdout(starts_with("com\t626\n").and(ends_with("za\t1\n")));

    Ok(())
}

#[test]
fn test_toplevel_1k_counts_top5() -> Result<(), Box<dyn Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--max-items").arg("5").arg(test_file);
    cmd.assert()
        .success()
        .stdout(starts_with("com\t626\n").and(ends_with("cn\t25\n")));

    Ok(())
}

#[test]
fn test_toplevel_1k_counts_sortby_count() -> Result<(), Box<dyn Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-m")
        .arg("5")
        .arg("-s")
        .arg("count")
        .arg(test_file);
    cmd.assert()
        .success()
        .stdout(starts_with("com\t626\n").and(ends_with("cn\t25\n")));

    Ok(())
}

#[test]
fn test_toplevel_1k_counts_sortby_key() -> Result<(), Box<dyn Error>> {
    let test_file = "tests/data/toplevel-1k.txt";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("--sort-by").arg("key").arg(test_file);
    cmd.assert()
        .success()
        .stdout(starts_with("ae\t1\n").and(ends_with("za\t1\n")));

    Ok(())
}

#[test]
fn test_incorrect_file() -> Result<(), Box<dyn Error>> {
    let test_file = "no_test_file_here";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg(test_file);
    cmd.assert()
        .failure()
        .stderr(contains("No such file or directory"));

    Ok(())
}
