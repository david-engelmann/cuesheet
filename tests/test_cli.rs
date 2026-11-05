//! The command line discipline, ported from the landed
//! Python suite test for test (CU-10 and CU-11).

use std::path::Path;

use sha2::{Digest, Sha256};

mod common;
use common::{MINI, run_cli, test_dir, write};

const CONTEXT: &str =
    "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n";

fn digest(path: &Path) -> String {
    format!(
        "{:x}",
        Sha256::digest(std::fs::read(path).expect("read input"))
    )
}

fn nonblank_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .collect()
}

#[test]
fn the_sheet_command_prints_the_sheet_and_exits_zero() {
    let dir = test_dir("cli");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(&dir, "p.context", CONTEXT);
    let result = run_cli(
        &[
            "sheet",
            catalogue.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 0);
    assert!(result.stdout_text().starts_with("sheet mini p\n"));
    assert!(result.stdout_text().contains("cue alpha\n"));
    assert!(result.stdout_text().contains("cue beta\n"));
    assert_eq!(result.stderr_text(), "");
}

#[test]
fn a_malformed_input_exits_two_with_one_error_line() {
    let dir = test_dir("cli");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(&dir, "p.context", "context mini\n");
    let result = run_cli(
        &[
            "sheet",
            catalogue.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
    assert_eq!(nonblank_lines(&result.stderr_text()).len(), 1);
    assert!(!result.stderr_text().contains("Traceback"));
}

#[test]
fn an_unreadable_input_exits_two_with_one_error_line() {
    let dir = test_dir("cli");
    let context = write(&dir, "p.context", CONTEXT);
    let missing = dir.join("absent.catalogue");
    let result = run_cli(
        &[
            "sheet",
            missing.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 2);
    assert_eq!(nonblank_lines(&result.stderr_text()).len(), 1);
    assert!(!result.stderr_text().contains("Traceback"));
}

#[test]
fn generating_a_sheet_changes_neither_input() {
    let dir = test_dir("cli");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(&dir, "p.context", CONTEXT);
    let before = (digest(&catalogue), digest(&context));
    let result = run_cli(
        &[
            "sheet",
            catalogue.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 0);
    let after = (digest(&catalogue), digest(&context));
    assert_eq!(before, after);
}
