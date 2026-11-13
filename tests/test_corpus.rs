//! The committed CLI corpus (CU-26). Its expectations are
//! sourced from the oracle for functional cases, from clap
//! for the presentation cases whose only difference is help
//! or malformed invocation text, and from the port itself
//! for the list family CU-30 states in the port's own voice.
//! The per source counts are pinned, so the split cannot
//! silently weaken. The partition property of CU-14 is also
//! exercised here over generated catalogue and context pairs.

use std::collections::BTreeMap;

use cuesheet::catalogue::parse_catalogue;
use cuesheet::context::parse_context;
use cuesheet::pytext::PyStr;
use cuesheet::sheet::{build_gaps, build_sheet};

mod common;
use common::{load_corpus, run_corpus_case};

#[test]
fn the_corpus_matches_its_sourced_expectations() {
    let cases = load_corpus();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &cases {
        *counts.entry(case.source.as_str()).or_insert(0) += 1;
    }
    assert_eq!(cases.len(), 148, "the corpus holds 137 cases");
    assert_eq!(counts.get("oracle"), Some(&95), "oracle sourced cases");
    assert_eq!(counts.get("clap"), Some(&50), "clap presentation cases");
    assert_eq!(counts.get("port"), Some(&3), "port voiced cases");
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| {
            run_corpus_case(case)
                .err()
                .map(|why| format!("{}: {why}", case.name))
        })
        .collect();
    assert!(failures.is_empty(), "corpus failures: {failures:#?}");
}

/// The shapes one token carries for the branch census of
/// the committed corpus (CU-44, CU-45).
fn branch_shapes(token: &str) -> Vec<&'static str> {
    let mut shapes = Vec::new();
    if token == "-h" {
        shapes.push("exact short");
    }
    if token == "--help" {
        shapes.push("exact long");
    }
    let long_abbreviation = |text: &str| {
        text.starts_with("--") && (3..=5).contains(&text.len()) && "--help".starts_with(text)
    };
    if long_abbreviation(token) {
        shapes.push("long abbreviation");
    }
    if let Some(eq) = token.find('=')
        && long_abbreviation(&token[..eq])
    {
        shapes.push("long abbreviation with equals");
    }
    if token.starts_with("-h=") {
        shapes.push("short equals head");
    }
    if token.starts_with("-h")
        && token.chars().count() > 2
        && !token.contains('=')
        && !token[1..].starts_with('-')
    {
        shapes.push("short bundle");
    }
    if token.starts_with("--help=") {
        shapes.push("valued long help");
    }
    let negative_number = {
        let mut chars = token.chars();
        match chars.next() {
            Some('-') => {
                let rest: Vec<char> = chars.collect();
                !rest.is_empty()
                    && rest[0] != '-'
                    && (rest.iter().all(|point| point.is_numeric())
                        || match rest.iter().position(|point| *point == '.') {
                            Some(dot) => {
                                let (head, tail) = (&rest[..dot], &rest[dot + 1..]);
                                !tail.is_empty()
                                    && head.iter().all(|point| point.is_numeric())
                                    && tail.iter().all(|point| point.is_numeric())
                            }
                            None => false,
                        })
            }
            _ => false,
        }
    };
    if negative_number {
        shapes.push("negative number");
    }
    let recognized_option_form = {
        let head = token.split('=').next().unwrap_or(token);
        head == "-h"
            || head == "--help"
            || (head.starts_with("--") && "--help".starts_with(head))
            || token.starts_with("-h")
    };
    if token.contains(' ') && !recognized_option_form {
        shapes.push("space carrying positional");
    }
    if token == "-" {
        shapes.push("lone dash");
    }
    if token == "--" {
        shapes.push("separator");
    }
    shapes
}

/// The branch census of the committed corpus (CU-44,
/// CU-45). Each case is counted once for a branch and a
/// position when any token of its invocation carries the
/// branch's shape, the root position standing ahead of the
/// command name and the subcommand position behind it, in
/// the manner of the census the Round 8 record carries. The
/// pin asserts the presence of cases by branch and by
/// position at the floors CU-44 states, and asserts no
/// expectation of any case.
#[test]
fn the_corpus_covers_the_classification_branches() {
    let cases = load_corpus();
    let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut abbreviation_lengths: std::collections::BTreeSet<usize> =
        std::collections::BTreeSet::new();
    for case in &cases {
        let command_index = case
            .argv
            .iter()
            .position(|token| ["sheet", "gaps", "catalogue"].contains(&token.as_str()));
        let mut seen: std::collections::BTreeSet<(&str, &str)> = std::collections::BTreeSet::new();
        for (index, token) in case.argv.iter().enumerate() {
            let position = match command_index {
                Some(command) if index > command => "subcommand",
                _ => "root",
            };
            for shape in branch_shapes(token) {
                seen.insert((shape, position));
                if shape == "long abbreviation" {
                    abbreviation_lengths.insert(token.chars().count());
                }
            }
        }
        for entry in seen {
            *counts.entry(entry).or_insert(0) += 1;
        }
    }
    let floor = |shape: &str, position: &str, minimum: usize| {
        let found = counts.get(&(shape, position)).copied().unwrap_or(0);
        assert!(
            found >= minimum,
            "the corpus holds {found} cases for the {shape} branch at the {position} \
             position, below the floor of {minimum} CU-44 states"
        );
    };
    for shape in [
        "exact short",
        "exact long",
        "long abbreviation",
        "long abbreviation with equals",
        "short equals head",
        "short bundle",
        "valued long help",
    ] {
        floor(shape, "root", 2);
        floor(shape, "subcommand", 2);
    }
    for shape in ["negative number", "space carrying positional", "lone dash"] {
        floor(shape, "subcommand", 2);
    }
    floor("separator", "subcommand", 2);
    floor("separator", "root", 1);
    assert!(
        abbreviation_lengths.len() >= 2,
        "the long abbreviations of the corpus span one length only, \
         where CU-44 states at least two distinct lengths"
    );
}

/// A small deterministic generator, so the generated pairs
/// are the same on every run and need no stored fixture.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

#[test]
fn the_two_views_partition_generated_catalogues() {
    let mut rng = Lcg(0xC0FFEE);
    for round in 0..60 {
        let action_count = 1 + rng.below(5);
        let mut catalogue_text = String::from(
            "catalogue gen\nfact tools set-of read, edit, run\nfact window values inside, outside\nfact last values none, read\n",
        );
        let mut action_names = Vec::new();
        for index in 0..action_count {
            let name = format!("act{index}");
            action_names.push(name.clone());
            catalogue_text.push_str(&format!("action {name}\n"));
            let test_count = 1 + rng.below(3);
            for _ in 0..test_count {
                match rng.below(3) {
                    0 => {
                        let token = ["read", "edit", "run"][rng.below(3) as usize];
                        let test = ["has", "lacks"][rng.below(2) as usize];
                        catalogue_text.push_str(&format!("requires tools {test} {token}\n"));
                    }
                    1 => {
                        let value = ["inside", "outside"][rng.below(2) as usize];
                        let test = ["is", "is-not"][rng.below(2) as usize];
                        catalogue_text.push_str(&format!("requires window {test} {value}\n"));
                    }
                    _ => {
                        let value = ["none", "read"][rng.below(2) as usize];
                        let test = ["is", "is-not"][rng.below(2) as usize];
                        catalogue_text.push_str(&format!("requires last {test} {value}\n"));
                    }
                }
            }
            catalogue_text.push_str(&format!("fires fire {index}\nfires-in-turn turn {index}\n"));
        }
        let mut context_text = String::from("context gen point\n");
        if rng.below(4) > 0 {
            let mut tokens = Vec::new();
            for token in ["read", "edit", "run"] {
                if rng.below(2) == 0 {
                    tokens.push(token);
                }
            }
            if tokens.is_empty() {
                tokens.push("read");
            }
            context_text.push_str(&format!("fact tools = {}\n", tokens.join(", ")));
        }
        if rng.below(4) > 0 {
            let value = ["inside", "outside"][rng.below(2) as usize];
            context_text.push_str(&format!("fact window = {value}\n"));
        }
        if rng.below(4) > 0 {
            let value = ["none", "read"][rng.below(2) as usize];
            context_text.push_str(&format!("fact last = {value}\n"));
        }
        let catalogue = parse_catalogue(&catalogue_text, &PyStr::from_str("gen.catalogue"))
            .unwrap_or_else(|e| panic!("round {round} catalogue refused: {}", e.message()));
        let context = parse_context(&context_text, &PyStr::from_str("gen.context"), &catalogue)
            .unwrap_or_else(|e| panic!("round {round} context refused: {}", e.message()));
        let sheet: Vec<String> = build_sheet(&catalogue, &context)
            .into_iter()
            .map(|cue| cue.name)
            .collect();
        let gaps = build_gaps(&catalogue, &context);
        let gap_names: Vec<String> = gaps.iter().map(|gap| gap.name.clone()).collect();
        let mut joined = sheet.clone();
        joined.extend(gap_names.iter().cloned());
        joined.sort_unstable();
        let mut expected = action_names.clone();
        expected.sort_unstable();
        assert_eq!(joined, expected, "round {round} partition");
        assert!(
            sheet.iter().all(|name| !gap_names.contains(name)),
            "round {round} overlap"
        );
        for gap in &gaps {
            assert!(!gap.blocked.is_empty(), "round {round} empty gap block");
        }
    }
}
