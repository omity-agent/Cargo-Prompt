use core::iter;
use std::{env, fs, path::Path, process::Command};
fn exercise(directory: &Path) {
    let manifest = directory.join("Cargo.toml");
    let source = directory.join("fixture.rs");
    fs::write(
        &manifest,
        "[package]\nname = \"agent-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\npath = \"fixture.rs\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(&source, include_str!("repairable.rs")).unwrap();
    let binary = Path::new(env!("CARGO_BIN_EXE_cargo-prompt"));
    let existing = env::var_os("PATH").unwrap();
    let search = env::join_paths(
        iter::once(binary.parent().unwrap())
            .map(Path::to_path_buf)
            .chain(env::split_paths(&existing)),
    )
    .unwrap();
    let output = Command::new(env!("CARGO"))
        .args([
            "prompt",
            "--message-format",
            "short",
            "--message-format=human",
            "--workspace=false",
            "--all-targets",
            "--fix",
            "--allow-no-vcs",
            "--allow-dirty",
            "--broken-code",
            "--config",
            "term.color=\"never\"",
            "--manifest-path",
        ])
        .arg(&manifest)
        .args(["--", "--warn", "clippy::missing_safety_doc"])
        .env("PATH", search)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let formatted = fs::read_to_string(&source).unwrap();
    assert!(!formatted.contains("mut number"));
    assert!(formatted.contains("pub fn repaired() -> i32 {\n"));
    let warnings = String::from_utf8(output.stdout).unwrap();
    assert!(warnings.starts_with("<Warning>\n"));
    assert!(warnings.contains("<missing_safety_doc>"));
    assert!(warnings.contains("<Snippet "));
    for omitted in ["for further information visit", "implied by", "to override"] {
        assert!(!warnings.contains(omitted), "{warnings}");
    }
    for (text, expected) in [
        ("pub fn broken() { let _: u8 = \"mismatch\"; }\n", "<E0308>"),
        ("let item = 1;\n", "expected item"),
    ] {
        fs::write(&source, text).unwrap();
        let failure = Command::new(binary)
            .arg("--manifest-path")
            .arg(&manifest)
            .output()
            .unwrap();
        assert_eq!(failure.status.code(), Some(101_i32));
        let errors = String::from_utf8(failure.stdout).unwrap();
        assert!(errors.starts_with("<Error>\n"));
        assert!(errors.contains(expected));
        assert!(!errors.contains("for a full list of items that can appear in modules, see"));
    }
    fs::write(&source, "pub fn clean() {}\n").unwrap();
    fs::write(directory.join("rustfmt.toml"), "max_width = false\n").unwrap();
    let formatting_failure = Command::new(binary)
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(formatting_failure.status.success());
    assert!(
        String::from_utf8(formatting_failure.stderr)
            .unwrap()
            .contains("cargo fmt failed")
    );
}
#[test]
fn cargo_dispatch_formats_sources_and_preserves_clippy_status() {
    let root = env::temp_dir().join("cargo-prompt-e2e");
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::tempdir_in(&root).unwrap();
    exercise(directory.path());
    directory.close().unwrap();
    fs::remove_dir(&root).unwrap();
}
