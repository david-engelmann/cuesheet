//! The context reading, ported from the landed Python
//! suite test for test (CU-1 and CU-4).

use std::collections::BTreeSet;

use cuesheet::catalogue::{Catalogue, parse_catalogue};
use cuesheet::context::{Context, Recorded, parse_context};
use cuesheet::errors::CuesheetError;
use cuesheet::pytext::PyStr;

mod common;
use common::{FULL_CONTEXT, MINI};

fn catalogue() -> Catalogue {
    parse_catalogue(MINI, &PyStr::from_str("mini.catalogue")).expect("catalogue parses")
}

fn parse(text: &str) -> Result<Context, CuesheetError> {
    parse_context(text, &PyStr::from_str("bad.context"), &catalogue())
}

fn message(text: &str) -> String {
    match parse(text) {
        Ok(_) => panic!("the context parsed"),
        Err(error) => error.message().to_display(),
    }
}

#[test]
fn a_valid_context_records_each_fact() {
    let context = parse_context(
        FULL_CONTEXT,
        &PyStr::from_str("start.context"),
        &catalogue(),
    )
    .expect("context parses");
    assert_eq!(context.harness, "mini");
    assert_eq!(context.decision_point, "start");
    assert_eq!(
        context.facts["tools"],
        Recorded::Tokens(BTreeSet::from(["read".to_string(), "edit".to_string()]))
    );
    assert_eq!(
        context.facts["window"],
        Recorded::Scalar("inside".to_string())
    );
    assert_eq!(context.facts["last"], Recorded::Scalar("read".to_string()));
}

#[test]
fn a_context_may_record_a_subset_of_the_facts() {
    let context = parse_context(
        "context mini start\nfact tools = read\n",
        &PyStr::from_str("sub.context"),
        &catalogue(),
    )
    .expect("context parses");
    assert_eq!(context.facts.len(), 1);
    assert!(context.facts.contains_key("tools"));
}

#[test]
fn a_missing_header_is_refused() {
    assert!(parse("fact tools = read\n").is_err());
}

#[test]
fn a_header_with_no_decision_point_is_refused() {
    assert!(parse("context mini\nfact tools = read\n").is_err());
}

#[test]
fn a_fact_the_catalogue_never_declares_is_refused() {
    let text = FULL_CONTEXT.replace("fact last = read", "fact weather = calm");
    assert!(message(&text).contains("weather"));
}

#[test]
fn a_fact_recorded_twice_is_refused() {
    let text = format!("{FULL_CONTEXT}fact window = outside\n");
    assert!(message(&text).contains("window"));
}

#[test]
fn a_recorded_value_outside_the_declared_values_is_refused() {
    let text = FULL_CONTEXT.replace("fact window = inside", "fact window = sideways");
    assert!(parse(&text).is_err());
}

#[test]
fn a_recorded_token_outside_the_declared_tokens_is_refused() {
    let text = FULL_CONTEXT.replace("fact tools = read, edit", "fact tools = read, fly");
    assert!(message(&text).contains("fly"));
}

#[test]
fn a_line_matching_no_form_is_refused() {
    let text = format!("{FULL_CONTEXT}wander off\n");
    assert!(parse(&text).is_err());
}

#[test]
fn a_harness_mismatch_is_refused_naming_both_harnesses() {
    let text = FULL_CONTEXT.replace("context mini start", "context other start");
    let message = message(&text);
    assert!(message.contains("other"));
    assert!(message.contains("mini"));
}
