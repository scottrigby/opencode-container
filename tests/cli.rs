use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("opencode-container").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("opencode-container").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_completion_bash() {
    let mut cmd = Command::cargo_bin("opencode-container").unwrap();
    cmd.args(["completion", "--bash"]);
    cmd.assert().success();
}

#[test]
fn test_completion_zsh() {
    let mut cmd = Command::cargo_bin("opencode-container").unwrap();
    cmd.args(["completion", "--zsh"]);
    cmd.assert().success();
}

#[test]
fn test_projects_no_panic() {
    let mut cmd = Command::cargo_bin("opencode-container").unwrap();
    cmd.arg("projects");
    // May succeed or fail depending on XDG dirs, but should not panic
    let output = cmd.output().unwrap();
    // Exit 0 is OK (no projects found), or other exit codes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("thread panicked"));
}
