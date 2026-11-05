//! The set reading, ported from the landed Python suite
//! test for test (CU-19).

use cuesheet::catalogue::{Catalogue, RuleTest, parse_catalogue};
use cuesheet::context::parse_context;
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_gaps, build_sheet, format_gaps, format_sheet};

const BASE: &str = "catalogue sets\nfact tools set-of read, edit\nfact window values inside, outside\naction alpha\nrequires tools has read\nfires alpha fires text\nfires-in-turn alpha turn text\naction beta\nrequires tools has edit\nrequires window is inside\nfires beta fires text\nfires-in-turn beta turn text\n";
const CONTEXT: &str = "context sets start\nfact tools = read\nfact window = inside\n";
const FULL_CONTEXT: &str = "context sets start\nfact tools = read, edit\nfact window = inside\n";

fn views(catalogue_text: &str, context_text: &str) -> (String, String) {
    let catalogue = parse_catalogue(catalogue_text, &PyStr::from_str("sets.catalogue"))
        .expect("catalogue parses");
    let context = parse_context(context_text, &PyStr::from_str("sets.context"), &catalogue)
        .expect("context parses");
    (
        format_sheet(&context, &build_sheet(&catalogue, &context)),
        format_gaps(&context, &build_gaps(&catalogue, &context)),
    )
}

fn catalogue_error(text: &str) -> String {
    match parse_catalogue(text, &PyStr::from_str("bad.catalogue")) {
        Ok(_) => panic!("the catalogue parsed"),
        Err(error) => error.message().to_display(),
    }
}

#[test]
fn a_repeated_option_in_a_values_list_reads_as_written_once() {
    let repeated = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside, inside, outside",
    );
    assert_eq!(views(&repeated, CONTEXT), views(BASE, CONTEXT));
}

#[test]
fn a_repeated_token_in_a_set_of_list_reads_as_written_once() {
    let repeated = BASE.replace(
        "fact tools set-of read, edit",
        "fact tools set-of read, edit, edit",
    );
    assert_eq!(views(&repeated, CONTEXT), views(BASE, CONTEXT));
}

#[test]
fn a_repeated_token_in_a_recorded_set_reads_as_written_once() {
    let repeated = FULL_CONTEXT.replace("fact tools = read, edit", "fact tools = read, edit, read");
    assert_eq!(views(BASE, &repeated), views(BASE, FULL_CONTEXT));
}

#[test]
fn whitespace_around_a_comma_is_no_part_of_an_option_or_token() {
    let snug = BASE.replace(
        "fact tools set-of read, edit",
        "fact tools set-of read,edit",
    );
    let loose = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside , outside",
    );
    let snug_context = FULL_CONTEXT.replace("fact tools = read, edit", "fact tools = read ,edit");
    assert_eq!(views(&snug, CONTEXT), views(BASE, CONTEXT));
    assert_eq!(views(&loose, CONTEXT), views(BASE, CONTEXT));
    assert_eq!(views(BASE, &snug_context), views(BASE, FULL_CONTEXT));
}

#[test]
fn an_empty_option_in_a_fact_declaration_is_refused() {
    let text = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside,, outside",
    );
    assert!(catalogue_error(&text).contains("window"));
}

#[test]
fn an_empty_token_in_a_set_of_list_is_refused() {
    let text = BASE.replace(
        "fact tools set-of read, edit",
        "fact tools set-of read, edit,",
    );
    assert!(catalogue_error(&text).contains("tools"));
}

#[test]
fn an_empty_token_in_a_recorded_set_is_refused() {
    let catalogue: Catalogue =
        parse_catalogue(BASE, &PyStr::from_str("sets.catalogue")).expect("catalogue parses");
    let text = FULL_CONTEXT.replace("fact tools = read, edit", "fact tools = read, edit,");
    let error = parse_context(&text, &PyStr::from_str("bad.context"), &catalogue)
        .expect_err("the context is refused");
    assert!(error.message().to_display().contains("tools"));
}

#[test]
fn a_repeated_passing_test_prints_its_evidence_line_once() {
    let repeated = BASE.replace(
        "action alpha\nrequires tools has read\n",
        "action alpha\nrequires tools has read\nrequires tools has read\n",
    );
    let (sheet, gaps) = views(&repeated, CONTEXT);
    let expected = views(BASE, CONTEXT);
    assert_eq!(sheet, expected.0);
    assert_eq!(gaps, expected.1);
    assert_eq!(
        sheet
            .matches("evidence tools has read, recorded read")
            .count(),
        1
    );
}

#[test]
fn a_repeated_failing_test_prints_its_blocked_line_once() {
    let repeated = BASE.replace(
        "action beta\nrequires tools has edit\n",
        "action beta\nrequires tools has edit\nrequires tools has edit\n",
    );
    let (sheet, gaps) = views(&repeated, CONTEXT);
    let expected = views(BASE, CONTEXT);
    assert_eq!(sheet, expected.0);
    assert_eq!(gaps, expected.1);
    assert_eq!(
        gaps.matches("blocked tools has edit, recorded read")
            .count(),
        1
    );
}

#[test]
fn the_parser_records_a_repeated_test_once_in_first_order() {
    let text = BASE.replace(
        "action alpha\nrequires tools has read\n",
        "action alpha\nrequires window is inside\nrequires tools has read\nrequires window is inside\n",
    );
    let catalogue =
        parse_catalogue(&text, &PyStr::from_str("sets.catalogue")).expect("catalogue parses");
    let stored = &catalogue.actions["alpha"].tests;
    assert_eq!(
        stored
            .iter()
            .map(|test| (test.fact.as_str(), test.test.as_str(), test.value.as_str()))
            .collect::<Vec<_>>(),
        vec![("window", "is", "inside"), ("tools", "has", "read")]
    );
    let (sheet, _) = views(&text, CONTEXT);
    let evidence: Vec<&str> = sheet
        .lines()
        .filter(|line| line.contains("evidence"))
        .collect();
    assert_eq!(
        evidence,
        [
            "evidence window is inside, recorded inside",
            "evidence tools has read, recorded read"
        ]
    );
}

fn triples(tests: &[RuleTest]) -> Vec<(&str, &str, &str)> {
    tests
        .iter()
        .map(|test| (test.fact.as_str(), test.test.as_str(), test.value.as_str()))
        .collect()
}

#[test]
fn two_actions_stating_the_same_test_each_keep_their_own() {
    let text = BASE.replace(
        "action beta\nrequires tools has edit\n",
        "action beta\nrequires tools has read\n",
    );
    let catalogue =
        parse_catalogue(&text, &PyStr::from_str("sets.catalogue")).expect("catalogue parses");
    assert_eq!(
        triples(&catalogue.actions["alpha"].tests),
        [("tools", "has", "read")]
    );
    assert_eq!(
        triples(&catalogue.actions["beta"].tests),
        [("tools", "has", "read"), ("window", "is", "inside")]
    );
    let (sheet, _) = views(&text, CONTEXT);
    assert_eq!(
        sheet
            .matches("evidence tools has read, recorded read")
            .count(),
        2
    );
}
