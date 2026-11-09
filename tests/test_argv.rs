//! The per branch pins of the argument classification (CU-48).
//! Each test drives the published scan over constructed tokens
//! and asserts the outcome the contract states for one branch,
//! the kind of the outcome and the payload the branch
//! constructs. Every expectation is derived from the
//! obligations that state the branch, and the mutation runs of
//! CU-49 stand in the round record as the proof the pins
//! discriminate.

use cuesheet::argv::{Cmd, Scan, Slot, scan};
use cuesheet::pytext::PyStr;

fn tokens(texts: &[&str]) -> Vec<PyStr> {
    texts.iter().map(|text| PyStr::from_str(text)).collect()
}

fn one(text: &str) -> Slot {
    Slot::One(PyStr::from_str(text))
}

fn expect_fire(texts: &[&str], cmd: Option<Cmd>) {
    match scan(&tokens(texts)) {
        Scan::FireHelp(found) => {
            assert_eq!(found, cmd, "the command help fires for {texts:?}")
        }
        _ => panic!("expected help to fire for {texts:?}"),
    }
}

fn expect_assigned(texts: &[&str], cmd: Cmd, slots: Vec<Slot>) {
    match scan(&tokens(texts)) {
        Scan::Assigned {
            cmd: found,
            slots: got,
        } => {
            assert_eq!(found, cmd, "the assigned command for {texts:?}");
            assert_eq!(got, slots, "the assigned slots for {texts:?}");
        }
        _ => panic!("expected an assignment for {texts:?}"),
    }
}

fn expect_hand_none(texts: &[&str]) {
    match scan(&tokens(texts)) {
        Scan::HandToClap { replacement: None } => {}
        _ => panic!("expected a plain hand to clap for {texts:?}"),
    }
}

fn expect_hand_replacement(texts: &[&str], index: usize, argument: &str) {
    match scan(&tokens(texts)) {
        Scan::HandToClap {
            replacement: Some((at, named)),
        } => {
            assert_eq!(at, index, "the replacement index for {texts:?}");
            assert_eq!(
                named,
                PyStr::from_str(argument),
                "the replacement argument for {texts:?}"
            );
        }
        _ => panic!("expected a hand to clap with a replacement for {texts:?}"),
    }
}

// The arms of classify, driven through the scan on minimal
// streams.

#[test]
fn classify_exact_short_help_fires() {
    expect_fire(&["-h"], None);
}

#[test]
fn classify_exact_long_help_fires() {
    expect_fire(&["--help"], None);
}

#[test]
fn classify_long_abbreviation_fires() {
    expect_fire(&["--h"], None);
    expect_fire(&["--he"], None);
}

#[test]
fn classify_long_abbreviation_behind_a_subcommand_fires() {
    expect_fire(&["gaps", "--hel"], Some(Cmd::Gaps));
}

#[test]
fn classify_long_equals_head_refuses_with_the_argument() {
    expect_hand_replacement(&["--help=x"], 0, "x");
}

#[test]
fn classify_short_equals_head_refuses_with_the_argument() {
    expect_hand_replacement(&["-h=x"], 0, "x");
}

#[test]
fn classify_long_abbreviation_with_equals_refuses() {
    expect_hand_replacement(&["--h=y"], 0, "y");
}

#[test]
fn classify_long_abbreviation_with_equals_and_an_empty_argument_refuses() {
    expect_hand_replacement(&["--h="], 0, "");
}

#[test]
fn classify_degenerate_long_prefix_with_equals_refuses() {
    expect_hand_replacement(&["--=x"], 0, "x");
}

#[test]
fn classify_unrecognized_equals_head_is_a_no_action_optional() {
    expect_hand_none(&["--foo=bar"]);
}

#[test]
fn classify_unrecognized_long_is_a_no_action_optional() {
    expect_hand_none(&["--bogus"]);
}

#[test]
fn classify_unrecognized_short_is_a_no_action_optional() {
    expect_hand_none(&["-x"]);
}

#[test]
fn classify_lone_dash_stands_as_a_positional() {
    expect_assigned(&["sheet", "-", "b"], Cmd::Sheet, vec![one("-"), one("b")]);
}

#[test]
fn classify_negative_number_stands_as_a_positional() {
    expect_assigned(&["sheet", "-1", "b"], Cmd::Sheet, vec![one("-1"), one("b")]);
    expect_assigned(
        &["gaps", "a", "-1.5"],
        Cmd::Gaps,
        vec![one("a"), one("-1.5")],
    );
}

#[test]
fn classify_negative_number_leading_dot_stands_as_a_positional() {
    expect_assigned(
        &["sheet", "-.5", "b"],
        Cmd::Sheet,
        vec![one("-.5"), one("b")],
    );
}

#[test]
fn classify_negative_number_trailing_dot_is_no_number() {
    expect_hand_none(&["sheet", "-1.", "b"]);
}

#[test]
fn classify_space_carrying_token_stands_as_a_positional() {
    expect_assigned(
        &["sheet", "-a b", "c"],
        Cmd::Sheet,
        vec![one("-a b"), one("c")],
    );
}

#[test]
fn classify_plain_token_stands_as_a_positional() {
    expect_assigned(&["catalogue", "a b"], Cmd::Catalogue, vec![one("a b")]);
}

// The outcomes of the walks, what the scan returns for a whole
// invocation.

#[test]
fn walk_help_fires_behind_each_subcommand() {
    expect_fire(&["sheet", "-h"], Some(Cmd::Sheet));
    expect_fire(&["gaps", "--help"], Some(Cmd::Gaps));
    expect_fire(&["catalogue", "-h"], Some(Cmd::Catalogue));
}

#[test]
fn walk_help_fires_after_a_complete_assignment() {
    expect_fire(&["sheet", "a", "b", "-h"], Some(Cmd::Sheet));
}

#[test]
fn walk_help_fires_after_an_extra_value() {
    expect_fire(&["sheet", "a", "b", "c", "-h"], Some(Cmd::Sheet));
}

#[test]
fn walk_sheet_assigns_two_paths() {
    expect_assigned(&["sheet", "a", "b"], Cmd::Sheet, vec![one("a"), one("b")]);
}

#[test]
fn walk_gaps_assigns_two_paths() {
    expect_assigned(&["gaps", "a", "b"], Cmd::Gaps, vec![one("a"), one("b")]);
}

#[test]
fn walk_catalogue_assigns_one_path() {
    expect_assigned(&["catalogue", "a"], Cmd::Catalogue, vec![one("a")]);
}

#[test]
fn walk_unknown_command_name_refuses() {
    expect_hand_none(&["bogus"]);
    expect_hand_none(&["bogus", "-h"]);
}

#[test]
fn walk_missing_paths_refuse() {
    expect_hand_none(&["sheet", "a"]);
    expect_hand_none(&["sheet"]);
}

#[test]
fn walk_extra_values_refuse() {
    expect_hand_none(&["sheet", "a", "b", "c"]);
    expect_hand_none(&["catalogue", "a", "b"]);
}

#[test]
fn walk_unknown_optional_at_the_root_refuses() {
    expect_hand_none(&["--bogus", "sheet", "a", "b"]);
}

#[test]
fn walk_unknown_optional_behind_a_subcommand_refuses() {
    expect_hand_none(&["sheet", "--bogus", "a", "b"]);
    expect_hand_none(&["gaps", "--bogus", "a", "b"]);
    expect_hand_none(&["catalogue", "--bogus", "a"]);
    expect_hand_none(&["sheet", "--version", "a", "b"]);
}

#[test]
fn walk_unknown_optional_after_a_complete_assignment_refuses() {
    expect_hand_none(&["sheet", "a", "b", "--bogus"]);
    expect_hand_none(&["gaps", "a", "b", "--bogus"]);
    expect_hand_none(&["catalogue", "a", "--bogus"]);
}

#[test]
fn walk_unknown_short_before_paths_behind_a_subcommand_refuses() {
    expect_hand_none(&["sheet", "-x", "a", "b"]);
    expect_hand_none(&["catalogue", "-x", "a"]);
}

#[test]
fn walk_empty_stream_refuses() {
    expect_hand_none(&[]);
}

#[test]
fn walk_replacement_index_counts_the_whole_stream() {
    expect_hand_replacement(&["sheet", "a", "b", "--help=x"], 3, "x");
}

#[test]
fn walk_replacement_behind_a_subcommand() {
    expect_hand_replacement(&["gaps", "--hel=x", "a", "b"], 1, "x");
}

// The outcomes of the help consumption, what a help tuple the
// walk meets does with its explicit argument.

#[test]
fn help_long_form_with_an_empty_value_refuses() {
    expect_hand_replacement(&["--help="], 0, "");
    expect_hand_replacement(&["sheet", "--help=", "a", "b"], 1, "");
}

#[test]
fn help_short_equals_with_an_empty_value_refuses() {
    expect_hand_replacement(&["-h="], 0, "");
}

#[test]
fn help_cluster_minus_remainder_refuses_the_whole_remainder() {
    expect_hand_replacement(&["-h-x"], 0, "-x");
}

#[test]
fn help_cluster_minus_after_an_h_refuses_the_remainder() {
    expect_hand_replacement(&["-hh-x"], 0, "-x");
}

#[test]
fn help_cluster_h_then_equals_refuses_the_value() {
    expect_hand_replacement(&["-hh=x"], 0, "x");
}

#[test]
fn help_cluster_h_then_equals_with_an_empty_text_refuses() {
    expect_hand_replacement(&["-hh="], 0, "");
}

#[test]
fn help_cluster_h_alone_fires() {
    expect_fire(&["-hh"], None);
}

#[test]
fn help_cluster_h_then_another_character_fires() {
    expect_fire(&["-hhx"], None);
}

#[test]
fn help_cluster_other_character_fires() {
    expect_fire(&["-hx"], None);
    expect_fire(&["sheet", "-hx", "a", "b"], Some(Cmd::Sheet));
}

// The positions of the separator in the distribution.

#[test]
fn separator_ahead_of_the_command_name_refuses() {
    expect_hand_none(&["--", "sheet", "a", "b"]);
}

#[test]
fn separator_inside_a_slice_is_struck() {
    expect_assigned(
        &["sheet", "a", "--", "b"],
        Cmd::Sheet,
        vec![one("a"), one("b")],
    );
}

#[test]
fn separator_trailing_leaves_the_assignment_untouched() {
    expect_assigned(
        &["sheet", "a", "b", "--"],
        Cmd::Sheet,
        vec![one("a"), one("b")],
    );
}

#[test]
fn separator_never_stands_in_for_a_missing_path() {
    expect_hand_none(&["sheet", "a", "--"]);
}

#[test]
fn separator_second_separator_stands_as_a_value() {
    expect_assigned(
        &["sheet", "--", "--", "c"],
        Cmd::Sheet,
        vec![one("--"), one("c")],
    );
}

#[test]
fn separator_slice_of_one_separator_leaves_an_empty_list() {
    expect_assigned(
        &["sheet", "a", "--", "--"],
        Cmd::Sheet,
        vec![one("a"), Slot::Listed(vec![])],
    );
}

#[test]
fn separator_after_the_first_disarms_a_help_token() {
    expect_hand_none(&["--", "-h"]);
}
