//! The committed examples, ported from the landed Python
//! suite test for test (CU-12). The catalogue document pin
//! stands under CU-36.

use std::path::{Path, PathBuf};

use cuesheet::catalogue::{format_catalogue, parse_catalogue};
use cuesheet::context::parse_context;
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_gaps, build_sheet, format_gaps, format_sheet};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn read_example(name: &str) -> String {
    std::fs::read_to_string(examples().join(name)).expect("read example")
}

fn produce(context_name: &str) -> String {
    let catalogue = parse_catalogue(
        &read_example("stagehand.catalogue"),
        &PyStr::from_str("stagehand.catalogue"),
    )
    .expect("catalogue parses");
    let context = parse_context(
        &read_example(context_name),
        &PyStr::from_str(context_name),
        &catalogue,
    )
    .expect("context parses");
    format_sheet(&context, &build_sheet(&catalogue, &context))
}

fn produce_document() -> String {
    let catalogue = parse_catalogue(
        &read_example("stagehand.catalogue"),
        &PyStr::from_str("stagehand.catalogue"),
    )
    .expect("catalogue parses");
    format_catalogue(&catalogue)
}

fn produce_gaps(context_name: &str) -> String {
    let catalogue = parse_catalogue(
        &read_example("stagehand.catalogue"),
        &PyStr::from_str("stagehand.catalogue"),
    )
    .expect("catalogue parses");
    let context = parse_context(
        &read_example(context_name),
        &PyStr::from_str(context_name),
        &catalogue,
    )
    .expect("context parses");
    format_gaps(&context, &build_gaps(&catalogue, &context))
}

fn names<'a>(text: &'a str, prefix: &str) -> Vec<&'a str> {
    text.lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .collect()
}

#[test]
fn the_ordinary_example_regenerates_its_committed_sheet() {
    assert_eq!(produce("ordinary.context"), read_example("ordinary.sheet"));
}

#[test]
fn the_restricted_example_regenerates_its_committed_sheet() {
    assert_eq!(
        produce("restricted.context"),
        read_example("restricted.sheet")
    );
}

#[test]
fn the_example_sheets_list_exactly_the_available_actions() {
    assert_eq!(
        names(&produce("ordinary.context"), "cue "),
        [
            "ask-human",
            "edit-file",
            "read-file",
            "run-command",
            "search-corpus",
            "send-update"
        ]
    );
    assert_eq!(
        names(&produce("restricted.context"), "cue "),
        ["read-file", "search-corpus"]
    );
}

#[test]
fn the_ordinary_example_regenerates_its_committed_gaps_view() {
    assert_eq!(
        produce_gaps("ordinary.context"),
        read_example("ordinary.gaps")
    );
}

#[test]
fn the_restricted_example_regenerates_its_committed_gaps_view() {
    assert_eq!(
        produce_gaps("restricted.context"),
        read_example("restricted.gaps")
    );
}

#[test]
fn the_example_catalogue_regenerates_its_committed_document() {
    assert_eq!(produce_document(), read_example("stagehand.document"));
}

#[test]
fn the_example_gaps_name_exactly_the_actions_the_sheets_leave_off() {
    assert_eq!(
        names(&produce_gaps("ordinary.context"), "gap "),
        ["spawn-helper"]
    );
    let restricted = produce_gaps("restricted.context");
    assert_eq!(
        names(&restricted, "gap "),
        [
            "ask-human",
            "edit-file",
            "run-command",
            "send-update",
            "spawn-helper"
        ]
    );
    assert!(restricted.contains("blocked human-present is yes, not recorded\n"));
}
