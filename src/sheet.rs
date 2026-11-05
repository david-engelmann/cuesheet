//! The sheet and the gaps view, from one evaluation of the rules.

use crate::catalogue::{Action, Catalogue, RuleTest};
use crate::context::{Context, Recorded};

/// One available action with its evidence lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub name: String,
    pub fires: String,
    pub fires_in_turn: String,
    pub evidence: Vec<String>,
}

/// One unavailable action with its blocked lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Gap {
    pub name: String,
    pub blocked: Vec<String>,
}

/// Render a recorded value: a recorded set stands joined in
/// sorted order and a recorded value stands as itself.
fn render_recorded(recorded: &Recorded) -> String {
    match recorded {
        Recorded::Scalar(value) => value.clone(),
        Recorded::Tokens(tokens) => tokens
            .iter()
            .map(|token| token.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Whether one rule test passes against a recorded value. A
/// set test on a scalar and a value test on a set cannot arise,
/// since the catalogue refuses a test whose form mismatches the
/// kind of its fact at load; the pairings are answered as the
/// oracle's equality and membership reads answer them.
fn test_passes(rule_test: &RuleTest, recorded: &Recorded) -> bool {
    match (rule_test.test.as_str(), recorded) {
        ("is", Recorded::Scalar(value)) => value == &rule_test.value,
        ("is-not", Recorded::Scalar(value)) => value != &rule_test.value,
        ("has", Recorded::Tokens(tokens)) => tokens.contains(&rule_test.value),
        ("lacks", Recorded::Tokens(tokens)) => !tokens.contains(&rule_test.value),
        _ => false,
    }
}

/// Evaluate every rule test of an action against the context.
///
/// Returns one triple per test in catalogue order, holding the
/// test, the recorded value it was checked against, and whether
/// it passed. The recorded value is absent when the context
/// does not record the fact, and a test on an unrecorded fact
/// never passes under CU-6. Both views are computed from these
/// triples, so they cannot disagree about an action.
fn evaluate_tests<'a>(
    action: &'a Action,
    context: &'a Context,
) -> Vec<(&'a RuleTest, Option<&'a Recorded>, bool)> {
    let mut results = Vec::new();
    for rule_test in &action.tests {
        match context.facts.get(&rule_test.fact) {
            None => results.push((rule_test, None, false)),
            Some(recorded) => {
                let passed = test_passes(rule_test, recorded);
                results.push((rule_test, Some(recorded), passed));
            }
        }
    }
    results
}

/// The action names of a catalogue in sorted order, which is
/// code point order as it is byte order for UTF-8 text.
fn sorted_action_names(catalogue: &Catalogue) -> Vec<&str> {
    let mut names: Vec<&str> = catalogue.actions.keys().map(|name| name.as_str()).collect();
    names.sort_unstable();
    names
}

/// Return the cues available in this context, sorted by name.
pub fn build_sheet(catalogue: &Catalogue, context: &Context) -> Vec<Cue> {
    let mut cues = Vec::new();
    for name in sorted_action_names(catalogue) {
        let action = &catalogue.actions[name];
        let results = evaluate_tests(action, context);
        if results.iter().all(|(_, _, passed)| *passed) {
            let evidence = results
                .iter()
                .map(|(rule_test, recorded, _)| {
                    format!(
                        "evidence {} {} {}, recorded {}",
                        rule_test.fact,
                        rule_test.test,
                        rule_test.value,
                        render_recorded(recorded.expect("a passing test is recorded"))
                    )
                })
                .collect();
            cues.push(Cue {
                name: action.name.clone(),
                fires: action.fires.clone(),
                fires_in_turn: action.fires_in_turn.clone(),
                evidence,
            });
        }
    }
    cues
}

/// Render one sheet as deterministic text.
pub fn format_sheet(context: &Context, cues: &[Cue]) -> String {
    let mut lines = vec![format!(
        "sheet {} {}",
        context.harness, context.decision_point
    )];
    for cue in cues {
        lines.push(format!("cue {}", cue.name));
        lines.push(format!("fires {}", cue.fires));
        lines.push(format!("fires-in-turn {}", cue.fires_in_turn));
        lines.extend(cue.evidence.iter().cloned());
    }
    lines.join("\n") + "\n"
}

/// Return the gaps, the unavailable actions and why, by name.
pub fn build_gaps(catalogue: &Catalogue, context: &Context) -> Vec<Gap> {
    let mut gaps = Vec::new();
    for name in sorted_action_names(catalogue) {
        let action = &catalogue.actions[name];
        let results = evaluate_tests(action, context);
        if results.iter().all(|(_, _, passed)| *passed) {
            continue;
        }
        let mut blocked = Vec::new();
        for (rule_test, recorded, passed) in &results {
            if *passed {
                continue;
            }
            match recorded {
                None => blocked.push(format!(
                    "blocked {} {} {}, not recorded",
                    rule_test.fact, rule_test.test, rule_test.value
                )),
                Some(recorded) => blocked.push(format!(
                    "blocked {} {} {}, recorded {}",
                    rule_test.fact,
                    rule_test.test,
                    rule_test.value,
                    render_recorded(recorded)
                )),
            }
        }
        gaps.push(Gap {
            name: action.name.clone(),
            blocked,
        });
    }
    gaps
}

/// Render the gaps view as deterministic text.
pub fn format_gaps(context: &Context, gaps: &[Gap]) -> String {
    let mut lines = vec![format!(
        "gaps {} {}",
        context.harness, context.decision_point
    )];
    for gap in gaps {
        lines.push(format!("gap {}", gap.name));
        lines.extend(gap.blocked.iter().cloned());
    }
    lines.join("\n") + "\n"
}
