//! Sheet evaluation, ported from the landed Python suite
//! test for test (CU-5 to CU-9).

use cuesheet::catalogue::{Catalogue, parse_catalogue};
use cuesheet::context::parse_context;
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_sheet, format_sheet};

mod common;
use common::MINI;

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

fn sheet_names(context_text: &str, catalogue: &Catalogue) -> Vec<String> {
    let context = parse_context(context_text, &PyStr::from_str("c.context"), catalogue)
        .expect("context parses");
    build_sheet(catalogue, &context)
        .into_iter()
        .map(|cue| cue.name)
        .collect()
}

#[test]
fn every_passing_action_is_listed_and_no_other() {
    let names = sheet_names(
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
        &catalogue(),
    );
    assert_eq!(names, ["alpha", "beta"]);
}

#[test]
fn one_failing_test_in_a_conjunction_excludes_the_action() {
    let names = sheet_names(
        "context mini p\nfact tools = edit\nfact window = outside\nfact last = read\n",
        &catalogue(),
    );
    assert!(!names.iter().any(|name| name == "beta"));
    assert!(names.is_empty());
}

#[test]
fn a_missing_fact_makes_the_action_unavailable() {
    let names = sheet_names(
        "context mini p\nfact tools = edit\nfact last = read\n",
        &catalogue(),
    );
    assert!(!names.iter().any(|name| name == "beta"));
}

#[test]
fn a_missing_fact_is_not_assumed_from_the_other_tests() {
    let names = sheet_names(
        "context mini p\nfact window = inside\nfact last = read\n",
        &catalogue(),
    );
    assert!(names.is_empty());
}

#[test]
fn lacks_tests_read_the_recorded_facts() {
    let catalogue = delta_catalogue();
    let names = sheet_names(
        "context mini p\nfact tools = read\nfact last = none\n",
        &catalogue,
    );
    assert_eq!(names, ["alpha", "delta"]);
    let names = sheet_names(
        "context mini p\nfact tools = read\nfact last = read\n",
        &catalogue,
    );
    assert!(names.iter().any(|name| name == "delta"));
}

#[test]
fn an_is_not_test_lists_the_action_when_the_value_differs() {
    let catalogue = epsilon_catalogue();
    let context = parse_context(
        "context mini p\nfact window = outside\n",
        &PyStr::from_str("c.context"),
        &catalogue,
    )
    .expect("context parses");
    let cues = build_sheet(&catalogue, &context);
    assert_eq!(
        cues.iter().map(|cue| cue.name.as_str()).collect::<Vec<_>>(),
        ["epsilon"]
    );
    assert_eq!(
        cues[0].evidence,
        vec!["evidence window is-not inside, recorded outside".to_string()]
    );
}

#[test]
fn an_is_not_test_excludes_the_action_when_the_value_matches() {
    let catalogue = epsilon_catalogue();
    let names = sheet_names("context mini p\nfact window = inside\n", &catalogue);
    assert!(!names.iter().any(|name| name == "epsilon"));
    assert!(names.is_empty());
}

#[test]
fn a_cue_carries_its_fires_text_and_ordered_evidence() {
    let catalogue = catalogue();
    let context = parse_context(
        "context mini p\nfact tools = edit, read\nfact window = inside\nfact last = read\n",
        &PyStr::from_str("c.context"),
        &catalogue,
    )
    .expect("context parses");
    let cues = build_sheet(&catalogue, &context);
    let beta = cues
        .iter()
        .find(|cue| cue.name == "beta")
        .expect("beta is listed");
    assert_eq!(beta.fires, "beta fires text");
    assert_eq!(beta.fires_in_turn, "beta turn text");
    assert_eq!(
        beta.evidence,
        vec![
            "evidence tools has edit, recorded edit, read".to_string(),
            "evidence window is inside, recorded inside".to_string(),
        ]
    );
}

#[test]
fn sheet_order_and_input_order_independence() {
    let shuffled = "catalogue mini\nfact last values none, read\nfact window values inside, outside\nfact tools set-of read, edit\naction beta\nrequires tools has edit\nrequires window is inside\nfires beta fires text\nfires-in-turn beta turn text\naction alpha\nrequires tools has read\nfires alpha fires text\nfires-in-turn alpha turn text\n";
    let catalogue = catalogue();
    let other = parse_catalogue(shuffled, &PyStr::from_str("shuffled.catalogue"))
        .expect("catalogue parses");
    let one = parse_context(
        "context mini p\nfact tools = read, edit\nfact window = inside\nfact last = read\n",
        &PyStr::from_str("a.context"),
        &catalogue,
    )
    .expect("context parses");
    let two = parse_context(
        "context mini p\nfact last = read\nfact window = inside\nfact tools = edit, read\n",
        &PyStr::from_str("b.context"),
        &other,
    )
    .expect("context parses");
    let first = format_sheet(&one, &build_sheet(&catalogue, &one));
    let second = format_sheet(&two, &build_sheet(&other, &two));
    assert_eq!(first, second);
    assert_eq!(first, format_sheet(&one, &build_sheet(&catalogue, &one)));
    assert_eq!(
        build_sheet(&catalogue, &one)
            .iter()
            .map(|cue| cue.name.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
}

#[test]
fn an_empty_sheet_is_the_header_alone() {
    let catalogue = catalogue();
    let context = parse_context(
        "context mini p\nfact last = none\n",
        &PyStr::from_str("c.context"),
        &catalogue,
    )
    .expect("context parses");
    let cues = build_sheet(&catalogue, &context);
    assert!(cues.is_empty());
    assert_eq!(format_sheet(&context, &cues), "sheet mini p\n");
}
