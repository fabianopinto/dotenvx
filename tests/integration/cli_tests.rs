use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_keypair_generation() {
    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("keypair")
        .assert()
        .success()
        .stdout(predicate::str::contains("DOTENV_PUBLIC_KEY"))
        .stdout(predicate::str::contains("DOTENV_PRIVATE_KEY"));
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let temp = TempDir::new().unwrap();
    let env_file = temp.path().join(".env");

    fs::write(&env_file, "SECRET=my_secret_value\n").unwrap();

    // Encrypt
    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("encrypt")
        .arg("-f")
        .arg(&env_file)
        .assert()
        .success();

    let content = fs::read_to_string(&env_file).unwrap();
    assert!(content.contains("encrypted:"));
    assert!(content.contains("DOTENV_PUBLIC_KEY"));

    // Decrypt
    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("decrypt")
        .arg("-f")
        .arg(&env_file)
        .assert()
        .success();

    let content = fs::read_to_string(&env_file).unwrap();
    assert!(content.contains("my_secret_value"));
    assert!(!content.contains("encrypted:"));
}

#[test]
fn test_set_command() {
    let temp = TempDir::new().unwrap();
    let env_file = temp.path().join(".env");

    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("set")
        .arg("API_KEY")
        .arg("secret123")
        .arg("-f")
        .arg(&env_file)
        .assert()
        .success();

    let content = fs::read_to_string(&env_file).unwrap();
    assert!(content.contains("API_KEY="));
    assert!(content.contains("encrypted:"));
}

#[test]
fn test_get_command() {
    let temp = TempDir::new().unwrap();
    let env_file = temp.path().join(".env");

    fs::write(&env_file, "TEST_KEY=test_value\n").unwrap();

    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("get")
        .arg("TEST_KEY")
        .arg("-f")
        .arg(&env_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("TEST_KEY=test_value"));
}

#[test]
fn test_ls_command() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join(".env"), "TEST=value\n").unwrap();
    fs::write(temp.path().join(".env.local"), "LOCAL=value\n").unwrap();

    let mut cmd = Command::cargo_bin("dotenvx").unwrap();
    cmd.arg("ls")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(".env"))
        .stdout(predicate::str::contains(".env.local"));
}

#[test]
fn test_run_command() {
    let temp = TempDir::new().unwrap();
    let env_file = temp.path().join(".env");

    fs::write(&env_file, "TEST_VAR=hello\n").unwrap();

    let mut cmd = Command::cargo_bin("dotenvx").unwrap();

    #[cfg(not(target_os = "windows"))]
    cmd.arg("run")
        .arg("-f")
        .arg(&env_file)
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg("echo $TEST_VAR")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));

    #[cfg(target_os = "windows")]
    cmd.arg("run")
        .arg("-f")
        .arg(&env_file)
        .arg("--")
        .arg("cmd")
        .arg("/C")
        .arg("echo %TEST_VAR%")
        .assert()
        .success();
}
