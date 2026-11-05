//! The gaps view, ported from the landed Python suite
//! test for test (CU-14 to CU-16).

use std::path::{Path, PathBuf};

use cuesheet::catalogue::{Catalogue, parse_catalogue};
use cuesheet::context::{Context, parse_context};
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_gaps, build_sheet, format_gaps};

mod common;
use common::{MINI, run_cli, test_dir, write};

fn catalogue() -> Catalogue {
    parse_catalogue(MINI, &PyStr::from_str("mini.catalogue")).expect("catalogue parses")
}

fn delta_catalogue() -> Catalogue {
    let text = format!(
        "{MINI}action delta\nrequires tools lacks edit\nfires delta fires text\nfires-in-turn delta turn text\n"
    );
    parse_catalogue(&text, &PyStr::from_str("delta.catalogue")).expect("catalogue parses")
}

fn epsilon_catalogue() -> Catalogue {
    let text = format!(
        "{MINI}action epsilon\nrequires window is-not inside\nfires epsilon fires text\nfires-in-turn epsilon turn text\n"
    );
    parse_catalogue(&text, &PyStr::from_str("epsilon.catalogue")).expect("catalogue parses")
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn read_example(name: &str) -> String {
    std::fs::read_to_string(examples().join(name)).expect("read example")
}

fn divide(catalogue: &Catalogue, context_text: &str) -> (Context, Vec<String>, Vec<String>) {
    let context = parse_context(context_text, &PyStr::from_str("c.context"), catalogue)
        .expect("context parses");
    let sheet = build_sheet(catalogue, &context)
        .into_iter()
        .map(|cue| cue.name)
        .collect();
    let gaps = build_gaps(catalogue, &context)
        .into_iter()
        .map(|gap| gap.name)
        .collect();
    (context, sheet, gaps)
}

fn gaps_text(catalogue: &Catalogue, context_text: &str) -> String {
    let context = parse_context(context_text, &PyStr::from_str("c.context"), catalogue)
        .expect("context parses");
    format_gaps(&context, &build_gaps(catalogue, &context))
}

#[test]
fn a_gap_block_names_only_the_tests_that_did_not_pass() {
    let text = gaps_text(
        &catalogue(),
        "context mini p\nfact tools = read, edit\nfact window = outside\nfact last = read\n",
    );
    assert_eq!(
        text,
        "gaps mini p\ngap beta\nblocked window is inside, recorded outside\n"
    );
}

#[test]
fn every_failing_test_is_accounted_for_in_catalogue_order() {
    let text = gaps_text(
        &catalogue(),
        "context mini p\nfact tools = read\nfact window = outside\nfact last = read\n",
    );
    assert_eq!(
        text,
        "gaps mini p\ngap beta\nblocked tools has edit, recorded read\nblocked window is inside, recorded outside\n"
    );
}

#[test]
fn a_test_on_an_unrecorded_fact_takes_the_not_recorded_form() {
    let text = gaps_text(
        &catalogue(),
        "context mini p\nfact tools = read, edit\nfact last = read\n",
    );
    assert_eq!(
        text,
        "gaps mini p\ngap beta\nblocked window is inside, not recorded\n"
    );
}

#[test]
fn blocked_lines_cover_the_is_not_and_lacks_tests() {
    let catalogue = delta_catalogue();
    let text = gaps_text(
        &catalogue,
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
    );
    assert!(text.contains("gap delta\n"));
    assert!(text.contains("blocked tools lacks edit, recorded edit, read\n"));
    let catalogue = epsilon_catalogue();
    let text = gaps_text(&catalogue, "context mini p\nfact window = inside\n");
    assert!(text.contains("gap epsilon\n"));
    assert!(text.contains("blocked window is-not inside, recorded inside\n"));
}

#[test]
fn the_two_views_partition_the_catalogue() {
    let catalogue = catalogue();
    let contexts = [
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
        "context mini p\nfact tools = read\nfact window = outside\nfact last = read\n",
        "context mini p\nfact tools = edit\nfact last = none\n",
        "context mini p\nfact last = none\n",
    ];
    let mut all_actions: Vec<&str> = catalogue.actions.keys().map(|name| name.as_str()).collect();
    all_actions.sort_unstable();
    for context_text in contexts {
        let (_, sheet, gaps) = divide(&catalogue, context_text);
        let mut joined = sheet.clone();
        joined.extend(gaps.iter().cloned());
        joined.sort_unstable();
        assert_eq!(joined, all_actions);
        assert!(sheet.iter().all(|name| !gaps.contains(name)));
    }
}

#[test]
fn the_two_views_partition_the_example_contexts() {
    let catalogue = parse_catalogue(
        &read_example("stagehand.catalogue"),
        &PyStr::from_str("stagehand.catalogue"),
    )
    .expect("catalogue parses");
    let expected = [
        (
            "ordinary.context",
            vec![
                "ask-human",
                "edit-file",
                "read-file",
                "run-command",
                "search-corpus",
                "send-update",
            ],
            vec!["spawn-helper"],
        ),
        (
            "restricted.context",
            vec!["read-file", "search-corpus"],
            vec![
                "ask-human",
                "edit-file",
                "run-command",
                "send-update",
                "spawn-helper",
            ],
        ),
    ];
    for (context_name, sheet_names, gap_names) in expected {
        let context = parse_context(
            &read_example(context_name),
            &PyStr::from_str(context_name),
            &catalogue,
        )
        .expect("context parses");
        assert_eq!(
            build_sheet(&catalogue, &context)
                .iter()
                .map(|cue| cue.name.as_str())
                .collect::<Vec<_>>(),
            sheet_names
        );
        assert_eq!(
            build_gaps(&catalogue, &context)
                .iter()
                .map(|gap| gap.name.as_str())
                .collect::<Vec<_>>(),
            gap_names
        );
    }
}

#[test]
fn no_gap_block_ever_stands_empty() {
    let catalogue = parse_catalogue(
        &read_example("stagehand.catalogue"),
        &PyStr::from_str("stagehand.catalogue"),
    )
    .expect("catalogue parses");
    for context_name in ["ordinary.context", "restricted.context"] {
        let context = parse_context(
            &read_example(context_name),
            &PyStr::from_str(context_name),
            &catalogue,
        )
        .expect("context parses");
        for gap in build_gaps(&catalogue, &context) {
            assert!(!gap.blocked.is_empty());
        }
    }
}

#[test]
fn when_nothing_is_unavailable_the_gaps_view_is_its_header_alone() {
    let text = gaps_text(
        &catalogue(),
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
    );
    assert_eq!(text, "gaps mini p\n");
}

#[test]
fn the_gaps_command_prints_and_exits_zero() {
    let dir = test_dir("gaps");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(
        &dir,
        "p.context",
        "context mini p\nfact tools = read\nfact window = outside\nfact last = read\n",
    );
    let result = run_cli(
        &[
            "gaps",
            catalogue.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 0);
    assert_eq!(
        result.stdout_text(),
        "gaps mini p\ngap beta\nblocked tools has edit, recorded read\nblocked window is inside, recorded outside\n"
    );
    assert_eq!(result.stderr_text(), "");
}

#[test]
fn the_gaps_command_with_no_gaps_prints_the_header_alone() {
    let dir = test_dir("gaps");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(
        &dir,
        "p.context",
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
    );
    let result = run_cli(
        &[
            "gaps",
            catalogue.to_str().expect("path text"),
            context.to_str().expect("path text"),
        ],
        &dir,
    );
    assert_eq!(result.code, 0);
    assert_eq!(result.stdout_text(), "gaps mini p\n");
}

#[test]
fn refusal_parity_for_a_malformed_catalogue() {
    let dir = test_dir("gaps");
    let catalogue = write(&dir, "bad.catalogue", "catalogue mini\nwander off\n");
    let context = write(&dir, "p.context", "context mini p\nfact last = none\n");
    let catalogue = catalogue.to_str().expect("path text");
    let context = context.to_str().expect("path text");
    let sheet = run_cli(&["sheet", catalogue, context], &dir);
    let gaps = run_cli(&["gaps", catalogue, context], &dir);
    assert_eq!(sheet.code, 2);
    assert_eq!(gaps.code, 2);
    assert!(gaps.stdout.is_empty());
    assert_eq!(gaps.stderr, sheet.stderr);
    assert!(!gaps.stderr_text().contains("Traceback"));
}

#[test]
fn refusal_parity_for_a_malformed_context() {
    let dir = test_dir("gaps");
    let catalogue = write(&dir, "mini.catalogue", MINI);
    let context = write(&dir, "p.context", "context mini\n");
    let catalogue = catalogue.to_str().expect("path text");
    let context = context.to_str().expect("path text");
    let sheet = run_cli(&["sheet", catalogue, context], &dir);
    let gaps = run_cli(&["gaps", catalogue, context], &dir);
    assert_eq!(sheet.code, 2);
    assert_eq!(gaps.code, 2);
    assert!(gaps.stdout.is_empty());
    assert_eq!(gaps.stderr, sheet.stderr);
    assert!(!gaps.stderr_text().contains("Traceback"));
}

#[test]
fn refusal_parity_for_an_unreadable_file() {
    let dir = test_dir("gaps");
    let context = write(&dir, "p.context", "context mini p\nfact last = none\n");
    let missing = dir.join("absent.catalogue");
    let missing = missing.to_str().expect("path text");
    let context = context.to_str().expect("path text");
    let sheet = run_cli(&["sheet", missing, context], &dir);
    let gaps = run_cli(&["gaps", missing, context], &dir);
    assert_eq!(sheet.code, 2);
    assert_eq!(gaps.code, 2);
    assert_eq!(gaps.stderr, sheet.stderr);
    let stderr = gaps.stderr_text();
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn the_gaps_view_is_deterministic_and_order_independent() {
    let catalogue = catalogue();
    let context_text =
        "context mini p\nfact tools = read\nfact window = outside\nfact last = read\n";
    let first = gaps_text(&catalogue, context_text);
    let second = gaps_text(&catalogue, context_text);
    assert_eq!(first, second);
    let shuffled = parse_context(
        "context mini p\nfact last = read\nfact window = outside\nfact tools = read\n",
        &PyStr::from_str("s.context"),
        &catalogue,
    )
    .expect("context parses");
    assert_eq!(
        format_gaps(&shuffled, &build_gaps(&catalogue, &shuffled)),
        first
    );
}
