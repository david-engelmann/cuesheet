//! The catalogue reading, ported from the landed Python
//! suite test for test (CU-2 and CU-3).

use cuesheet::catalogue::parse_catalogue;
use cuesheet::pytext::PyStr;

mod common;
use common::MINI;

fn parse(text: &str) -> Result<cuesheet::catalogue::Catalogue, cuesheet::errors::CuesheetError> {
    parse_catalogue(text, &PyStr::from_str("bad.catalogue"))
}

fn message(text: &str) -> String {
    match parse(text) {
        Ok(_) => panic!("the catalogue parsed"),
        Err(error) => error.message().to_display(),
    }
}

#[test]
fn a_valid_catalogue_declares_facts_and_actions() {
    let catalogue = parse_catalogue(MINI, &PyStr::from_str("mini.catalogue")).expect("parses");
    assert_eq!(catalogue.harness, "mini");
    assert_eq!(catalogue.facts["tools"].kind, "set-of");
    assert_eq!(
        catalogue.facts["tools"].options,
        vec!["read".to_string(), "edit".to_string()]
    );
    assert_eq!(catalogue.facts["window"].kind, "values");
    let beta = &catalogue.actions["beta"];
    assert_eq!(
        beta.tests
            .iter()
            .map(|test| (test.fact.as_str(), test.test.as_str(), test.value.as_str()))
            .collect::<Vec<_>>(),
        vec![("tools", "has", "edit"), ("window", "is", "inside")]
    );
    assert_eq!(beta.fires, "beta fires text");
    assert_eq!(beta.fires_in_turn, "beta turn text");
}

#[test]
fn a_missing_header_is_refused() {
    assert!(parse("fact tools set-of read\n").is_err());
}

#[test]
fn a_fact_declared_twice_is_refused() {
    let text = MINI.replace(
        "fact window values inside, outside\n",
        "fact window values inside, outside\nfact window values inside, outside\n",
    );
    assert!(message(&text).contains("window"));
}

#[test]
fn an_action_declared_twice_is_refused() {
    let text =
        format!("{MINI}action alpha\nrequires tools has read\nfires again\nfires-in-turn again\n");
    assert!(message(&text).contains("alpha"));
}

#[test]
fn a_rule_naming_an_undeclared_fact_is_refused() {
    let text = MINI.replace("requires window is inside", "requires weather is inside");
    assert!(parse(&text).is_err());
}

#[test]
fn a_test_whose_form_mismatches_the_fact_kind_is_refused() {
    let text = MINI.replace("requires tools has edit", "requires tools is edit");
    assert!(parse(&text).is_err());
    let text = MINI.replace("requires window is inside", "requires window has inside");
    assert!(parse(&text).is_err());
}

#[test]
fn a_tested_value_outside_the_declared_options_is_refused() {
    let text = MINI.replace("requires window is inside", "requires window is sideways");
    assert!(parse(&text).is_err());
}

#[test]
fn an_action_with_no_rule_test_is_refused() {
    let text = MINI.replace("requires tools has read\n", "");
    assert!(message(&text).contains("alpha"));
}

#[test]
fn an_action_missing_its_fires_line_is_refused() {
    let text = MINI.replace("fires alpha fires text\n", "");
    assert!(parse(&text).is_err());
}

#[test]
fn an_action_missing_its_fires_in_turn_line_is_refused() {
    let text = MINI.replace("fires-in-turn beta turn text\n", "");
    assert!(parse(&text).is_err());
}

#[test]
fn a_line_matching_no_form_is_refused() {
    let text = format!("{MINI}wander off\n");
    assert!(parse(&text).is_err());
}

#[test]
fn a_requires_line_before_any_action_is_refused() {
    let text = MINI.replacen("action alpha", "requires tools has read\naction alpha", 1);
    assert!(parse(&text).is_err());
}

#[test]
fn fires_text_after_a_tab_separator_is_kept_whole() {
    let text = MINI.replace("fires alpha fires text", "fires\talpha fires text");
    let catalogue = parse_catalogue(&text, &PyStr::from_str("tab.catalogue")).expect("parses");
    assert_eq!(catalogue.actions["alpha"].fires, "alpha fires text");
}

#[test]
fn a_fires_line_with_no_text_is_refused() {
    let text = MINI.replace("fires alpha fires text\n", "fires\n");
    assert!(parse(&text).is_err());
}
