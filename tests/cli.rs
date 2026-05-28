use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

fn glyf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_glyf"))
}

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn xdg_env() -> (Command, PathBuf) {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("glyf-test-{}-{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create test temp dir");
    let mut cmd = glyf();
    cmd.env("XDG_DATA_HOME", &dir);
    (cmd, dir)
}

#[test]
fn bare_hex_uplus_lookup() {
    let output = glyf().arg("U+0041").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("LATIN CAPITAL LETTER A"));
}

#[test]
fn bare_hex_0x_lookup() {
    let output = glyf().arg("0x0041").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("A"));
}

#[test]
fn bare_hex_no_prefix_lookup() {
    let output = glyf().arg("0041").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("A"));
}

#[test]
fn bare_single_ascii_char_lookup() {
    let output = glyf().arg("A").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("LATIN CAPITAL LETTER A"));
}

#[test]
fn bare_single_non_ascii_char_lookup() {
    let output = glyf().arg("😀").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("GRINNING FACE"));
}

#[test]
fn bare_name_search_fallback() {
    let output = glyf().arg("musical").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(!stdout.is_empty());
}

#[test]
fn explicit_codepoint_flag() {
    let output = glyf()
        .args(["-c", "U+0041"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("A"));
}

#[test]
fn explicit_search_flag() {
    let output = glyf()
        .args(["-s", "musical"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(!stdout.is_empty());
}

#[test]
fn not_found_exits_nonzero() {
    let output = glyf()
        .arg("XYZZY_NONEXISTENT_12345")
        .output()
        .expect("failed to run glyf");
    assert!(!output.status.success());
}

#[test]
fn help_flag() {
    let output = glyf().arg("--help").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("glyf"));
}

#[test]
fn version_flag() {
    let output = glyf()
        .arg("--version")
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
}

#[test]
fn list_all() {
    let output = glyf().arg("-l").output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("A"));
    assert!(stdout.contains("😀"));
}

#[test]
fn pick_records_frecency() {
    let (mut cmd, dir) = xdg_env();
    let output = cmd
        .arg("-p")
        .arg("U+0041")
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success(), "pick exited with error");
    let frec_path = dir.join("glyf").join("frecency.bin");
    assert!(frec_path.exists(), "frecency.bin should exist after pick");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pick_then_empty_search_shows_frecency_hit() {
    let (mut cmd, dir) = xdg_env();
    cmd.arg("-p")
        .arg("U+0041")
        .output()
        .expect("failed to run glyf");
    let output = glyf()
        .env("XDG_DATA_HOME", &dir)
        .arg("")
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("A"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn format_plain() {
    let output = glyf()
        .args(["-c", "U+0041", "-f", "plain"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert_eq!(stdout.trim(), "A");
}

#[test]
fn format_pretty() {
    let output = glyf()
        .args(["-c", "U+0041", "-f", "pretty"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("U+0041"));
}

#[test]
fn format_tsv() {
    let output = glyf()
        .args(["-c", "U+0041", "-f", "tsv"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.starts_with("65"));
}

#[test]
fn limit_results() {
    let output = glyf()
        .args(["-s", "a", "-n", "3"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() <= 3);
}

#[test]
fn sort_by_name() {
    let output = glyf()
        .args(["-s", "musical", "--sort", "name"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
}

#[test]
fn sort_by_codepoint() {
    let output = glyf()
        .args(["-s", "a", "--sort", "codepoint"])
        .output()
        .expect("failed to run glyf");
    assert!(output.status.success());
}

#[test]
fn bad_codepoint_exits_nonzero() {
    let output = glyf()
        .arg("-c")
        .arg("ZZZZZ")
        .output()
        .expect("failed to run glyf");
    assert!(!output.status.success());
}

#[test]
fn empty_args_shows_help() {
    let output = glyf().output().expect("failed to run glyf");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout not utf-8");
    assert!(stdout.contains("glyf"));
}

#[test]
fn bad_format_exits_nonzero() {
    let output = glyf()
        .args(["-c", "U+0041", "-f", "badger"])
        .output()
        .expect("failed to run glyf");
    assert!(!output.status.success());
}
