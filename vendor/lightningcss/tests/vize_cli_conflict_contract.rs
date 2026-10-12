//! Whole Clap4 conflict contract, alongside the preserved original CLI authority.
use std::process::Command;

#[test]
fn complete_targets_browserslist_conflict() {
  let output = Command::new(env!("CARGO_BIN_EXE_lightningcss"))
    .args(["--targets", "defaults", "--browserslist"])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(2));
  assert_eq!(output.stdout, b"");
  assert_eq!(output.stderr, b"error: the argument '--targets <TARGETS>' cannot be used with '--browserslist'\n\nUsage: lightningcss --targets <TARGETS> [INPUT_FILE]...\n\nFor more information, try '--help'.\n");
}
