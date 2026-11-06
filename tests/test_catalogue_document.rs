//! The catalogue document and its discipline (CU-32 to
//! CU-35), ported from the Python-era pins of the design and
//! extended with the separator readings the one positional
//! probe of this round recorded.

use cuesheet::catalogue::{format_catalogue, parse_catalogue};
use cuesheet::pytext::PyStr;

mod common;
use common::{DeadStream, run_cli, run_cli_dead, test_dir, write};

const CATALOGUE: &str = "\
catalogue audit
fact tools set-of read, edit, search
fact window values outside, inside
fact last values none, read
action beta
requires tools has edit
requires window is inside
requires tools has edit
fires beta fires text
fires-in-turn beta turn text
action alpha
requires tools lacks search
requires last is-not read
fires alpha fires text
fires-in-turn alpha turn text
";

const DOCUMENT: &str = "\
catalogue audit
fact tools set-of read, edit, search
fact window values outside, inside
fact last values none, read
action beta
evidence tools has edit
evidence window is inside
action alpha
evidence tools lacks search
evidence last is-not read
";

fn document(catalogue_text: &str) -> String {
    let catalogue =
        parse_catalogue(catalogue_text, &PyStr::from_str("audit.catalogue")).expect("parses");
    format_catalogue(&catalogue)
}

#[test]
fn the_catalogue_document_prints_the_stated_line_forms() {
    assert_eq!(document(CATALOGUE), DOCUMENT);
}

#[test]
fn the_document_blocks_stand_in_declaration_order() {
    let text = document(CATALOGUE);
    let facts: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("fact "))
        .map(|line| line.split_whitespace().nth(1).expect("fact name"))
        .collect();
    let actions: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("action "))
        .map(|line| line.split_whitespace().nth(1).expect("action name"))
        .collect();
    assert_eq!(facts, ["tools", "window", "last"]);
    assert_eq!(actions, ["beta", "alpha"]);
    let mut sorted_facts = facts.clone();
    sorted_facts.sort_unstable();
    let mut sorted_actions = actions.clone();
    sorted_actions.sort_unstable();
    assert_ne!(facts, sorted_facts);
    assert_ne!(actions, sorted_actions);
}

#[test]
fn a_repeated_rule_test_prints_its_evidence_line_once() {
    let text = document(CATALOGUE);
    assert_eq!(text.matches("evidence tools has edit\n").count(), 1);
}

#[test]
fn the_parser_records_a_repeated_option_once_in_first_order() {
    let text = CATALOGUE
        .replace(
            "fact tools set-of read, edit, search",
            "fact tools set-of edit, read, edit, search, read",
        )
        .replace(
            "fact window values outside, inside",
            "fact window values outside, inside, outside",
        );
    let catalogue = parse_catalogue(&text, &PyStr::from_str("audit.catalogue")).expect("parses");
    assert_eq!(catalogue.facts["tools"].options, ["edit", "read", "search"]);
    assert_eq!(catalogue.facts["window"].options, ["outside", "inside"]);
}

#[test]
fn a_repeated_option_in_a_values_list_reads_as_written_once() {
    let repeated = CATALOGUE.replace(
        "fact window values outside, inside",
        "fact window values outside, inside, outside",
    );
    assert_eq!(document(&repeated), DOCUMENT);
}

#[test]
fn a_repeated_option_in_a_set_of_list_reads_as_written_once() {
    let repeated = CATALOGUE.replace(
        "fact tools set-of read, edit, search",
        "fact tools set-of read, edit, read, search, edit",
    );
    assert_eq!(document(&repeated), DOCUMENT);
}

#[test]
fn a_header_only_catalogue_prints_its_header_alone() {
    assert_eq!(document("catalogue audit\n"), "catalogue audit\n");
}

#[test]
fn a_facts_only_catalogue_prints_its_facts_and_no_action_block() {
    let text =
        "catalogue audit\nfact tools set-of read, edit\nfact window values inside, outside\n";
    assert_eq!(
        document(text),
        "catalogue audit\nfact tools set-of read, edit\nfact window values inside, outside\n"
    );
}

#[test]
fn the_catalogue_command_prints_the_document_and_exits_zero() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let catalogue = catalogue.to_str().expect("path text");
    let result = run_cli(&["catalogue", catalogue], &dir);
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout_text(), DOCUMENT);
    assert!(result.stderr.is_empty());
}

#[test]
fn the_catalogue_command_takes_no_context_path() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let context = write(&dir, "audit.context", "context audit now\n");
    let catalogue = catalogue.to_str().expect("path text");
    let context = context.to_str().expect("path text");
    let result = run_cli(&["catalogue", catalogue, context], &dir);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
}

#[test]
fn a_malformed_catalogue_refuses_as_the_sheet_refuses_it() {
    let dir = test_dir("catalogue");
    let bad = CATALOGUE.replace(
        "fact last values none, read\n",
        "fact last values none, read\nfact last values none, read\n",
    );
    let catalogue = write(&dir, "bad.catalogue", &bad);
    let catalogue = catalogue.to_str().expect("path text");
    let missing_context = dir.join("absent.context");
    let missing_context = missing_context.to_str().expect("path text");
    let result = run_cli(&["catalogue", catalogue], &dir);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
    let stderr = result.stderr_text();
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(!stderr.contains("Traceback"));
    let sheet = run_cli(&["sheet", catalogue, missing_context], &dir);
    assert_eq!(sheet.code, 2);
    assert_eq!(sheet.stderr, result.stderr);
}

#[test]
fn an_unreadable_catalogue_exits_two_with_one_error_line() {
    let dir = test_dir("catalogue");
    let missing = dir.join("absent.catalogue");
    let missing = missing.to_str().expect("path text");
    let context = write(&dir, "audit.context", "context audit now\n");
    let context = context.to_str().expect("path text");
    let result = run_cli(&["catalogue", missing], &dir);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
    let stderr = result.stderr_text();
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(!stderr.contains("Traceback"));
    let sheet = run_cli(&["sheet", missing, context], &dir);
    assert_eq!(sheet.code, 2);
    assert_eq!(sheet.stderr, result.stderr);
}

#[test]
fn a_trailing_separator_after_the_path_assigns_it() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let catalogue = catalogue.to_str().expect("path text");
    let result = run_cli(&["catalogue", catalogue, "--"], &dir);
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout_text(), DOCUMENT);
}

#[test]
fn a_separator_before_the_path_assigns_it() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let catalogue = catalogue.to_str().expect("path text");
    let result = run_cli(&["catalogue", "--", catalogue], &dir);
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout_text(), DOCUMENT);
}

#[test]
fn the_separator_string_itself_is_a_path_after_a_separator() {
    let dir = test_dir("catalogue");
    write(&dir, "--", CATALOGUE);
    let result = run_cli(&["catalogue", "--", "--"], &dir);
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout_text(), DOCUMENT);
}

#[test]
fn a_second_value_after_a_separator_refuses() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let catalogue = catalogue.to_str().expect("path text");
    let result = run_cli(&["catalogue", "--", "--", catalogue], &dir);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
}

#[test]
fn the_document_print_to_a_dead_stream_answers_120() {
    let dir = test_dir("catalogue");
    let catalogue = write(&dir, "audit.catalogue", CATALOGUE);
    let catalogue = catalogue.to_str().expect("path text");
    let outcome = run_cli_dead(&["catalogue", catalogue], &dir, DeadStream::Stdout);
    assert_eq!(outcome.code, 120);
    assert_eq!(
        outcome.stderr_text(),
        "cuesheet could not print its document\n"
    );
}
