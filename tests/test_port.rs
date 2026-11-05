//! Port pins for the behaviours the landed Python suites
//! did not state as library calls (CU-22 to CU-30): the
//! argument scan at the binary surface, separator
//! distributions, list slots, path spellings at the open,
//! negative number tokens, dead streams at the measured
//! divide, the manifest placement, and source hygiene.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod common;
use common::{DeadStream, MINI, run_cli, run_cli_dead, test_dir, write, write_bytes};

const FULL: &str =
    "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n";
const LIST_LINE: &str =
    "cuesheet: an argument arrived as a list where a single path was expected\n";
const FAILED_PRINT_LINE: &str = "cuesheet could not print its document\n";
const UNCAUGHT_PRINT_LINE: &str = "cuesheet: the document print failed\n";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn pair(dir: &Path) -> (String, String) {
    let catalogue = write(dir, "c.catalogue", MINI);
    let context = write(dir, "c.context", FULL);
    (
        catalogue.to_str().expect("path text").to_string(),
        context.to_str().expect("path text").to_string(),
    )
}

// The separator distribution, CU-22 and CU-30.

#[test]
fn separators_around_two_paths_assign_normally() {
    let dir = test_dir("port-separators");
    let (catalogue, context) = pair(&dir);
    for argv in [
        vec!["sheet", catalogue.as_str(), context.as_str()],
        vec!["sheet", "--", catalogue.as_str(), context.as_str()],
        vec!["sheet", catalogue.as_str(), "--", context.as_str()],
        vec!["sheet", catalogue.as_str(), context.as_str(), "--"],
        vec!["gaps", "--", catalogue.as_str(), context.as_str()],
        vec!["gaps", catalogue.as_str(), context.as_str(), "--"],
    ] {
        let outcome = run_cli(&argv, &dir);
        assert_eq!(outcome.code, 0, "{argv:?}");
        assert!(outcome.stderr.is_empty(), "{argv:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_second_separator_is_an_ordinary_value() {
    let dir = test_dir("port-separators");
    let (_, context) = pair(&dir);
    let outcome = run_cli(&["sheet", "--", "--", context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file -- cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_list_in_the_context_slot_answers_the_list_line_at_one() {
    let dir = test_dir("port-list");
    let (catalogue, _) = pair(&dir);
    for argv in [
        vec!["sheet", catalogue.as_str(), "--", "--"],
        vec!["sheet", "--", catalogue.as_str(), "--"],
        vec!["gaps", catalogue.as_str(), "--", "--"],
    ] {
        let outcome = run_cli(&argv, &dir);
        assert_eq!(outcome.code, 1, "{argv:?}");
        assert!(outcome.stdout.is_empty(), "{argv:?}");
        assert_eq!(outcome.stderr_text(), LIST_LINE, "{argv:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_catalogue_refusal_answers_before_a_context_list_surfaces() {
    let dir = test_dir("port-list-order");
    let bad = write(&dir, "bad.catalogue", "catalogue mini\nwander off\n");
    let bad = bad.to_str().expect("path text");
    let outcome = run_cli(&["sheet", bad, "--", "--"], &dir);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr_text().contains("is malformed"));
    let outcome = run_cli(&["sheet", "absent.catalogue", "--", "--"], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file absent.catalogue cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_extra_value_preempts_the_list_family() {
    let dir = test_dir("port-list-extras");
    let (catalogue, _) = pair(&dir);
    let outcome = run_cli(&["sheet", catalogue.as_str(), "--", "--", "extra"], &dir);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stdout.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// Help precedence and the attached forms, CU-22.

#[test]
fn help_fires_where_a_help_token_stands() {
    let dir = test_dir("port-help");
    let (catalogue, context) = pair(&dir);
    for argv in [
        vec!["--help"],
        vec!["-h"],
        vec!["--he"],
        vec!["--h"],
        vec!["-hh"],
        vec!["-hhh"],
        vec!["-hx"],
        vec!["-hhhx"],
        vec!["-help"],
        vec!["--bogus", "--help"],
        vec!["sheet", "--help"],
        vec!["sheet", "-h"],
        vec!["sheet", "--he"],
        vec!["sheet", "--bogus", "--help"],
        vec!["sheet", catalogue.as_str(), context.as_str(), "--help"],
        vec!["sheet", "--help", "extra", "junk"],
        vec!["gaps", "--help"],
    ] {
        let outcome = run_cli(&argv, &dir);
        assert_eq!(outcome.code, 0, "{argv:?}");
        assert!(!outcome.stdout.is_empty(), "{argv:?}");
        assert!(outcome.stderr.is_empty(), "{argv:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn attached_help_arguments_refuse_at_two() {
    let dir = test_dir("port-help-attached");
    let (catalogue, context) = pair(&dir);
    for argv in [
        vec!["-h=x"],
        vec!["--help=x"],
        vec!["--help="],
        vec!["--=x"],
        vec!["-h="],
        vec!["-h-v"],
        vec!["-hh-v"],
        vec!["-hhh-v"],
        vec!["-h-h"],
        vec!["-hh=x"],
        vec!["-hh="],
        vec!["sheet", "-h=x", catalogue.as_str(), context.as_str()],
        vec!["sheet", "--help=x", catalogue.as_str(), context.as_str()],
        vec!["gaps", "-hh-v", catalogue.as_str(), context.as_str()],
    ] {
        let outcome = run_cli(&argv, &dir);
        assert_eq!(outcome.code, 2, "{argv:?}");
        assert!(outcome.stdout.is_empty(), "{argv:?}");
        assert!(!outcome.stderr.is_empty(), "{argv:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unknown_short_flag_before_two_paths_refuses_the_invocation() {
    let dir = test_dir("port-root-extras");
    let (catalogue, context) = pair(&dir);
    let outcome = run_cli(&["sheet", "-x", catalogue.as_str(), context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stdout.is_empty());
    assert!(
        outcome
            .stderr_text()
            .contains("Usage: cuesheet sheet <CATALOGUE> <CONTEXT>"),
        "{}",
        outcome.stderr_text()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// Values that look like flags and the negative number
// grammar, CU-22.

#[test]
fn negative_number_tokens_are_values() {
    let dir = test_dir("port-negative");
    let (_, context) = pair(&dir);
    for token in ["-1", "-1.5", "-.5", "-\u{661}\u{662}", "-\u{ff11}\u{ff12}"] {
        let outcome = run_cli(&["sheet", token, context.as_str()], &dir);
        assert_eq!(outcome.code, 2, "{token}");
        assert_eq!(
            outcome.stderr_text(),
            format!("error: the catalogue file {token} cannot be read\n")
        );
    }
    let outcome = run_cli(&["sheet", "-", context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file - cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn optional_shaped_tokens_are_not_values() {
    let dir = test_dir("port-negative-shapes");
    let (_, context) = pair(&dir);
    for token in ["-1_000", "-1.", "-1e3", "-\u{bd}", "-\u{b2}", "--1", "-a"] {
        let outcome = run_cli(&["sheet", token, context.as_str()], &dir);
        assert_eq!(outcome.code, 2, "{token}");
        assert!(outcome.stdout.is_empty(), "{token}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// Paths probed at the open, CU-25.

#[test]
fn folded_spellings_read_the_file_they_name() {
    let dir = test_dir("port-path-folds");
    write(&dir, "c.catalogue", MINI);
    write(&dir, "c.context", FULL);
    std::fs::create_dir_all(dir.join("sub")).expect("sub dir");
    for spelling in [
        "c.catalogue/",
        "c.catalogue/.",
        "./c.catalogue",
        "c.catalogue//",
        "sub/../c.catalogue",
        ".//c.catalogue",
    ] {
        let outcome = run_cli(&["sheet", spelling, "c.context"], &dir);
        assert_eq!(outcome.code, 0, "{spelling}");
        assert!(outcome.stdout_text().starts_with("sheet mini p\n"));
    }
    for spelling in [
        "c.context/",
        "c.context/.",
        "./c.context",
        "sub/../c.context",
    ] {
        let outcome = run_cli(&["sheet", "c.catalogue", spelling], &dir);
        assert_eq!(outcome.code, 0, "{spelling}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_cannot_read_line_names_the_path_as_written() {
    let dir = test_dir("port-path-names");
    let (_, context) = pair(&dir);
    for spelling in ["ghost.txt/", "ghost.txt/.", "./ghost.txt", "a//ghost.txt"] {
        let outcome = run_cli(&["sheet", spelling, context.as_str()], &dir);
        assert_eq!(outcome.code, 2, "{spelling}");
        assert_eq!(
            outcome.stderr_text(),
            format!("error: the catalogue file {spelling} cannot be read\n")
        );
    }
    let (catalogue, _) = pair(&dir);
    let outcome = run_cli(&["sheet", catalogue.as_str(), "./ghost.context"], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the context file ./ghost.context cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_directory_an_empty_path_and_a_non_utf8_body_cannot_be_read() {
    let dir = test_dir("port-path-refusals");
    let (catalogue, context) = pair(&dir);
    std::fs::create_dir_all(dir.join("sub")).expect("sub dir");
    let outcome = run_cli(&["sheet", "sub", context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file sub cannot be read\n"
    );
    let outcome = run_cli(&["sheet", "", context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file  cannot be read\n"
    );
    write_bytes(&dir, "bad.catalogue", b"catalogue mini\n\xff\n");
    let outcome = run_cli(&["sheet", "./bad.catalogue", context.as_str()], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file ./bad.catalogue cannot be read\n"
    );
    let outcome = run_cli(&["sheet", catalogue.as_str(), "sub"], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the context file sub cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn a_non_utf8_file_name_displays_with_surrogate_escapes() {
    let dir = test_dir("port-path-surrogate");
    let (_, context) = pair(&dir);
    let context_bytes = context.as_bytes();
    let outcome = common::run_cli_bytes(&[b"sheet", b"ghost\xff.catalogue", context_bytes], &dir);
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file ghost\\udcff.catalogue cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// Dead streams at the measured divide, CU-29.

fn sized_pair(dir: &Path, target: usize) -> (String, String) {
    // The fixed part of this sheet is 80 bytes and every
    // padding character of the fires text adds one byte.
    assert!(target >= 80, "the target must fit the fixed sheet");
    let padding = "x".repeat(target - 80);
    let catalogue = format!(
        "catalogue pad\nfact mode values on\naction only\nrequires mode is on\nfires {padding}\nfires-in-turn turn\n"
    );
    let catalogue = write(dir, "pad.catalogue", &catalogue);
    let context = write(dir, "pad.context", "context pad p\nfact mode = on\n");
    (
        catalogue.to_str().expect("path text").to_string(),
        context.to_str().expect("path text").to_string(),
    )
}

#[test]
fn a_document_under_the_buffer_on_dead_stdout_answers_120() {
    let dir = test_dir("port-dead-small");
    for target in [128usize, 8191] {
        let (catalogue, context) = sized_pair(&dir, target);
        let live = run_cli(&["sheet", catalogue.as_str(), context.as_str()], &dir);
        assert_eq!(live.code, 0);
        assert_eq!(live.stdout.len(), target);
        let outcome = run_cli_dead(
            &["sheet", catalogue.as_str(), context.as_str()],
            &dir,
            DeadStream::Stdout,
        );
        assert_eq!(outcome.code, 120, "target {target}");
        assert_eq!(outcome.stderr_text(), FAILED_PRINT_LINE, "target {target}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_document_at_the_buffer_on_dead_stdout_answers_one() {
    let dir = test_dir("port-dead-large");
    for target in [8192usize, 8193] {
        let (catalogue, context) = sized_pair(&dir, target);
        let live = run_cli(&["sheet", catalogue.as_str(), context.as_str()], &dir);
        assert_eq!(live.code, 0);
        assert_eq!(live.stdout.len(), target);
        let outcome = run_cli_dead(
            &["sheet", catalogue.as_str(), context.as_str()],
            &dir,
            DeadStream::Stdout,
        );
        assert_eq!(outcome.code, 1, "target {target}");
        assert_eq!(
            outcome.stderr_text(),
            UNCAUGHT_PRINT_LINE,
            "target {target}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dead_stderr_turns_refusals_and_usage_to_120_at_any_size() {
    let dir = test_dir("port-dead-stderr");
    let (catalogue, context) = pair(&dir);
    let bad = write(&dir, "bad.catalogue", "catalogue mini\nwander off\n");
    let bad = bad.to_str().expect("path text");
    let outcome = run_cli_dead(&["sheet", bad, context.as_str()], &dir, DeadStream::Stderr);
    assert_eq!(outcome.code, 120);
    assert!(outcome.stdout.is_empty());
    let long_name = "g".repeat(9000);
    let outcome = run_cli_dead(
        &["sheet", long_name.as_str(), context.as_str()],
        &dir,
        DeadStream::Stderr,
    );
    assert_eq!(outcome.code, 120);
    let outcome = run_cli_dead(&["sheet"], &dir, DeadStream::Stderr);
    assert_eq!(outcome.code, 120);
    let outcome = run_cli_dead(
        &["sheet", catalogue.as_str(), "--", "--"],
        &dir,
        DeadStream::Stderr,
    );
    assert_eq!(outcome.code, 120);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn help_on_dead_stdout_answers_120_and_success_ignores_dead_stderr() {
    let dir = test_dir("port-dead-help");
    let (catalogue, context) = pair(&dir);
    let outcome = run_cli_dead(&["--help"], &dir, DeadStream::Stdout);
    assert_eq!(outcome.code, 120);
    assert_eq!(outcome.stderr_text(), FAILED_PRINT_LINE);
    let outcome = run_cli_dead(
        &["sheet", catalogue.as_str(), context.as_str()],
        &dir,
        DeadStream::Stderr,
    );
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout_text().starts_with("sheet mini p\n"));
    let outcome = run_cli_dead(
        &["sheet", "absent.catalogue", context.as_str()],
        &dir,
        DeadStream::Stdout,
    );
    assert_eq!(outcome.code, 2);
    assert_eq!(
        outcome.stderr_text(),
        "error: the catalogue file absent.catalogue cannot be read\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// The manifest and the shipped sources, CU-27 as amended by
// CU-31.

fn manifest_sections(text: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(done) = current.take() {
                sections.push(done);
            }
            current = Some((trimmed.trim_matches(['[', ']']).to_string(), String::new()));
        } else if let Some((_, body)) = &mut current {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(done) = current.take() {
        sections.push(done);
    }
    sections
}

fn header_segments(header: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in header.trim().chars() {
        match quote {
            Some(close) if ch == close => quote = None,
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch == '.' => segments.push(std::mem::take(&mut current)),
            None => current.push(ch),
        }
    }
    segments.push(current);
    segments
        .into_iter()
        .map(|segment| segment.trim().to_string())
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn dependency_header(header: &str) -> Option<(&'static str, Option<String>)> {
    const KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    let segments = header_segments(header);
    let kind_at = segments
        .iter()
        .position(|segment| KINDS.contains(&segment.as_str()))?;
    let kind = KINDS
        .iter()
        .copied()
        .find(|kind| segments[kind_at] == *kind)?;
    let dotted = segments.get(kind_at + 1).cloned();
    Some((kind, dotted))
}

fn body_crate_names(body: &str, out: &mut BTreeSet<String>) {
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim().trim_matches(['"', '\'']);
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                out.insert(name.to_string());
            }
        }
    }
}

fn dependency_sets(text: &str) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let mut shipped = BTreeSet::new();
    let mut dev = BTreeSet::new();
    let mut build = BTreeSet::new();
    for (header, body) in manifest_sections(text) {
        let Some((kind, dotted)) = dependency_header(&header) else {
            continue;
        };
        let target = match kind {
            "dev-dependencies" => &mut dev,
            "build-dependencies" => &mut build,
            _ => &mut shipped,
        };
        if let Some(name) = dotted {
            target.insert(name);
            continue;
        }
        body_crate_names(&body, target);
    }
    (shipped, dev, build)
}

#[test]
fn the_manifest_places_its_dependencies_as_the_contract_states() {
    let text = std::fs::read_to_string(root().join("Cargo.toml")).expect("read manifest");
    let (shipped, dev, build) = dependency_sets(&text);
    assert_eq!(
        shipped,
        BTreeSet::from([
            "anyhow".to_string(),
            "clap".to_string(),
            "thiserror".to_string()
        ]),
        "the shipped set is exactly the set the contract allows"
    );
    assert_eq!(
        dev,
        BTreeSet::from(["serde_json".to_string(), "sha2".to_string()]),
        "the dev set is exactly the test only set the contract allows"
    );
    assert!(build.is_empty(), "no build dependencies stand");
    // The reader itself, over the manifest forms the contract
    // names, so the pin cannot pass by reading nothing.
    let (shipped, _, _) = dependency_sets("[dependencies.serde]\nversion = \"1\"\n");
    assert!(shipped.contains("serde"), "dotted table form must read");
    let (shipped, _, _) = dependency_sets("[target.'cfg(unix)'.dependencies]\nserde = \"1\"\n");
    assert!(shipped.contains("serde"), "target dotted form must read");
    let (shipped, _, _) =
        dependency_sets("[target.'cfg(unix)'.dependencies.serde_json]\nversion = \"1\"\n");
    assert!(
        shipped.contains("serde_json"),
        "target dotted per dependency form must read"
    );
    let (shipped, _, _) =
        dependency_sets("[dependencies.clap]\nversion = \"4\"\nfeatures = [\"derive\"]\n");
    assert_eq!(
        shipped,
        BTreeSet::from(["clap".to_string()]),
        "a per dependency table names its crate alone"
    );
    let (_, _, build) = dependency_sets("[build-dependencies]\nserde_json = \"1\"\n");
    assert!(build.contains("serde_json"), "build dependencies must read");
    let (_, dev, _) = dependency_sets("[dev-dependencies.serde]\nversion = \"1\"\n");
    assert!(dev.contains("serde"), "dotted dev form must read");
}

#[test]
fn no_python_files_stand_in_the_tree() {
    let root = root();
    let mut bad = Vec::new();
    fn walk(dir: &Path, root: &Path, bad: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().expect("type").is_dir() {
                if name == ".git" || name == "target" {
                    continue;
                }
                walk(&path, root, bad);
            } else if name.ends_with(".py") || name == "pyproject.toml" || name == "conftest.py" {
                bad.push(
                    path.strip_prefix(root)
                        .expect("under root")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    walk(&root, &root, &mut bad);
    assert!(bad.is_empty(), "python files remain: {bad:?}");
}

/// The crate roots one source text imports, resolving use
/// trees in grouped, nested, aliased, and glob forms, root
/// level groups and leading double colon paths, and bare
/// module imports.
fn imported_roots(text: &str) -> Vec<String> {
    fn first_segment(text: &str) -> &str {
        text.split([':', ';', '{', ' '])
            .next()
            .expect("a root stands")
    }
    let mut roots = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("use ") else {
            continue;
        };
        let rest = rest.trim_start();
        let rest = rest.strip_prefix("::").unwrap_or(rest);
        if let Some(group) = rest.strip_prefix('{') {
            let mut depth = 1usize;
            let mut end = group.len();
            for (at, ch) in group.char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = at;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let inner = &group[..end];
            let mut members = Vec::new();
            let mut member_depth = 0usize;
            let mut start = 0usize;
            for (at, ch) in inner.char_indices() {
                match ch {
                    '{' => member_depth += 1,
                    '}' => member_depth -= 1,
                    ',' if member_depth == 0 => {
                        members.push(&inner[start..at]);
                        start = at + 1;
                    }
                    _ => {}
                }
            }
            members.push(&inner[start..]);
            for member in members {
                let member = member.trim().trim_start_matches("::");
                let first = first_segment(member);
                if !first.is_empty() {
                    roots.push(first.to_string());
                }
            }
            continue;
        }
        let first = first_segment(rest);
        if !first.is_empty() {
            roots.push(first.to_string());
        }
    }
    roots
}

#[test]
fn the_import_scan_resolves_every_use_form() {
    assert_eq!(imported_roots("use serde_json::Value;\n"), ["serde_json"]);
    assert_eq!(
        imported_roots("use serde::{Serialize, de::DeserializeOwned};\n"),
        ["serde"]
    );
    assert_eq!(
        imported_roots("use serde::de::{self, DeserializeOwned};\n"),
        ["serde"]
    );
    assert_eq!(imported_roots("use sha2::Digest as _;\n"), ["sha2"]);
    assert_eq!(imported_roots("use clap::*;\n"), ["clap"]);
    assert_eq!(imported_roots("use serde;\n"), ["serde"]);
    assert_eq!(
        imported_roots("use {anyhow::Result, std::fmt};\n"),
        ["anyhow", "std"]
    );
    assert_eq!(
        imported_roots("use {std::{fmt, io}, anyhow::Result};\n"),
        ["std", "anyhow"]
    );
    assert_eq!(imported_roots("use ::anyhow::Result;\n"), ["anyhow"]);
}

#[test]
fn the_shipped_sources_never_import_the_serde_family() {
    let src = root().join("src");
    for entry in std::fs::read_dir(&src).expect("read src") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read source");
        for imported in imported_roots(&text) {
            assert!(
                imported != "serde" && imported != "serde_json",
                "{} imports {imported}",
                path.display()
            );
        }
    }
}

#[test]
fn anyhow_is_confined_to_the_binary_entry() {
    let src = root().join("src");
    for entry in std::fs::read_dir(&src).expect("read src") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read source");
        if path.file_name().and_then(|n| n.to_str()) == Some("main.rs") {
            continue;
        }
        for imported in imported_roots(&text) {
            assert_ne!(
                imported,
                "anyhow",
                "{} imports anyhow outside the binary entry",
                path.display()
            );
        }
    }
}
