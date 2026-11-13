use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_names_program_and_succeeds() {
    Command::cargo_bin("ctwinkle")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage: ctwinkle"))
        .stdout(contains("--help"))
        .stdout(contains("-C, --colour"))
        .stdout(contains("--christmas"))
        .stdout(contains("--version"));
}

#[test]
fn unknown_flag_fails() {
    Command::cargo_bin("ctwinkle")
        .unwrap()
        .arg("--nope")
        .assert()
        .failure()
        .stderr(contains("--nope"));
}

#[test]
fn unknown_colour_fails_before_terminal_changes() {
    Command::cargo_bin("ctwinkle")
        .unwrap()
        .args(["-C", "purple"])
        .assert()
        .failure()
        .stderr(contains("purple"))
        .stdout(predicates::str::is_empty());
}
