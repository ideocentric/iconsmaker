//! CLI contract for invocations that don't select any work.
//!
//! The key case here mirrors winget's install-and-launch validation: it runs
//! the installed binary with NO arguments in a directory that has no
//! icons.toml. That must print help and exit 0 — a non-zero exit is flagged as
//! Validation-Executable-Error. See src/main.rs "Bare invocation → help".

use std::path::PathBuf;
use std::process::Command;

/// A fresh, empty working directory (no icons.toml), removed on drop.
struct CleanDir(PathBuf);

impl CleanDir {
    fn new(tag: &str) -> Self {
        // Unique per test + process; no external randomness needed.
        let dir = std::env::temp_dir().join(format!("iconsmaker-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        CleanDir(dir)
    }
}

impl Drop for CleanDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_in(dir: &CleanDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_iconsmaker"))
        .args(args)
        .current_dir(&dir.0)
        .output()
        .expect("run iconsmaker")
}

#[test]
fn bare_launch_prints_help_and_exits_zero() {
    let dir = CleanDir::new("bare");
    let out = run_in(&dir, &[]);

    assert!(
        out.status.success(),
        "bare launch (no args, no icons.toml) must exit 0; got {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Usage:"),
        "bare launch must print help to stdout; got:\n{stdout}",
    );
}

#[test]
fn incomplete_flags_still_error() {
    // A partial/invalid selection is a real usage error and must stay non-zero
    // (the validation grid), unaffected by the bare-launch help shortcut.
    let dir = CleanDir::new("incomplete");
    let out = run_in(&dir, &["--snap"]); // packaging without --linux

    assert!(
        !out.status.success(),
        "incomplete flags must exit non-zero; got success",
    );
}
