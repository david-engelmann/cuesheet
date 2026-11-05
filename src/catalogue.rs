//! The action catalogue, the fixed offer of one harness family.

use std::collections::HashMap;

use crate::errors::{CuesheetError, malformed};
use crate::lines::meaningful_lines;
use crate::pytext::{PyStr, after_first_word, py_split, py_strip};

/// The kind word of a value fact, as the catalogue states it.
pub const VALUE: &str = "values";
/// The kind word of a set fact, as the catalogue states it.
pub const SET: &str = "set-of";

/// One declared fact, with the options it can carry. The
/// options stand as declared, repetitions included, because
/// the only reads the format makes of them are membership
/// reads, under which a repetition changes nothing (CU-19).
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub name: String,
    pub kind: String,
    pub options: Vec<String>,
}

/// One rule test of an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuleTest {
    pub fact: String,
    pub test: String,
    pub value: String,
}

/// One declared action, its distinct rule tests in first
/// occurrence order, and its two text lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub name: String,
    pub tests: Vec<RuleTest>,
    pub fires: String,
    pub fires_in_turn: String,
}

/// A parsed catalogue.
#[derive(Debug, Clone, PartialEq)]
pub struct Catalogue {
    pub harness: String,
    pub facts: HashMap<String, Fact>,
    pub actions: HashMap<String, Action>,
}

/// The tests that read a fact of the given kind, in the order
/// the oracle names them.
fn tests_for_kind(kind: &str) -> [&'static str; 2] {
    if kind == VALUE {
        ["is", "is-not"]
    } else {
        ["has", "lacks"]
    }
}

/// Split a declared list at its commas and strip each piece,
/// refusing the list when any piece is empty.
fn value_list(raw: &str, source: &PyStr, what: &str) -> Result<Vec<String>, CuesheetError> {
    let values: Vec<String> = raw
        .split(',')
        .map(|part| py_strip(part).to_string())
        .collect();
    if values.iter().any(|value| value.is_empty()) {
        return Err(malformed(format!(
            "{source} is malformed, the {what} carries an empty value"
        )));
    }
    Ok(values)
}

/// Refuse a rule test the catalogue does not admit, in the
/// oracle's order: an undeclared fact first, then a test whose
/// form mismatches the kind of the fact, then a tested value
/// outside the declared options.
fn check_test(
    fact: Option<&Fact>,
    test: &str,
    value: &str,
    source: &PyStr,
) -> Result<(), CuesheetError> {
    let Some(fact) = fact else {
        return Err(malformed(format!(
            "{source} is malformed, a rule test names a fact the catalogue never declares"
        )));
    };
    if !tests_for_kind(&fact.kind).contains(&test) {
        return Err(malformed(format!(
            "{source} is malformed, the test {test} does not read a fact of kind {}, in the rule for {}",
            fact.kind, fact.name
        )));
    }
    if !fact.options.iter().any(|option| option == value) {
        return Err(malformed(format!(
            "{source} is malformed, the test on {} names {value}, which the fact never declares",
            fact.name
        )));
    }
    Ok(())
}

struct Pending {
    name: String,
    tests: Vec<RuleTest>,
    fires: Option<String>,
    fires_in_turn: Option<String>,
}

/// Refuse or finish the action being built, in the oracle's
/// order: no rule test first, then a missing fires line, then
/// a missing fires-in-turn line. The tests stand deduped in
/// first occurrence order (CU-19).
fn finish_action(pending: Pending, source: &PyStr) -> Result<Action, CuesheetError> {
    let Pending {
        name,
        tests,
        fires,
        fires_in_turn,
    } = pending;
    if tests.is_empty() {
        return Err(malformed(format!(
            "{source} is malformed, the action {name} carries no rule test"
        )));
    }
    let Some(fires) = fires else {
        return Err(malformed(format!(
            "{source} is malformed, the action {name} carries no fires line"
        )));
    };
    let Some(fires_in_turn) = fires_in_turn else {
        return Err(malformed(format!(
            "{source} is malformed, the action {name} carries no fires-in-turn line"
        )));
    };
    let mut distinct: Vec<RuleTest> = Vec::new();
    for test in tests {
        if !distinct.contains(&test) {
            distinct.push(test);
        }
    }
    Ok(Action {
        name,
        tests: distinct,
        fires,
        fires_in_turn,
    })
}

/// Parse one catalogue or refuse it naming the defect.
pub fn parse_catalogue(text: &str, source: &PyStr) -> Result<Catalogue, CuesheetError> {
    let lines = meaningful_lines(text);
    if lines.is_empty() {
        return Err(malformed(format!(
            "{source} is malformed, it names no harness"
        )));
    }
    let head = py_split(lines[0]);
    if head.len() != 2 || head[0] != "catalogue" {
        return Err(malformed(format!(
            "{source} is malformed, the first line does not name the harness, it must read catalogue and a harness name"
        )));
    }
    let harness = head[1].to_string();
    let mut facts: HashMap<String, Fact> = HashMap::new();
    let mut actions: HashMap<String, Action> = HashMap::new();
    let mut pending: Option<Pending> = None;
    for line in &lines[1..] {
        let parts = py_split(line);
        if parts[0] == "fact" && parts.len() >= 4 && (parts[2] == VALUE || parts[2] == SET) {
            if pending.is_some() {
                return Err(malformed(format!(
                    "{source} is malformed, the fact {} is declared after the first action",
                    parts[1]
                )));
            }
            let name = parts[1].to_string();
            let kind = parts[2].to_string();
            if facts.contains_key(&name) {
                return Err(malformed(format!(
                    "{source} is malformed, the fact {name} is declared twice"
                )));
            }
            let options = value_list(&parts[3..].join(" "), source, &format!("fact {name}"))?;
            facts.insert(
                name.clone(),
                Fact {
                    name,
                    kind,
                    options,
                },
            );
        } else if parts[0] == "action" && parts.len() == 2 {
            if let Some(done_pending) = pending.take() {
                let done = finish_action(done_pending, source)?;
                actions.insert(done.name.clone(), done);
            }
            let name = parts[1].to_string();
            if actions.contains_key(&name) {
                return Err(malformed(format!(
                    "{source} is malformed, the action {name} is declared twice"
                )));
            }
            pending = Some(Pending {
                name,
                tests: Vec::new(),
                fires: None,
                fires_in_turn: None,
            });
        } else if parts[0] == "requires"
            && parts.len() == 4
            && let Some(current) = pending.as_mut()
        {
            check_test(facts.get(parts[1]), parts[2], parts[3], source)?;
            current.tests.push(RuleTest {
                fact: parts[1].to_string(),
                test: parts[2].to_string(),
                value: parts[3].to_string(),
            });
        } else if parts[0] == "fires"
            && parts.len() > 1
            && let Some(current) = pending.as_mut()
        {
            if current.fires.is_some() {
                return Err(malformed(format!(
                    "{source} is malformed, the action {} carries two fires lines",
                    current.name
                )));
            }
            current.fires = Some(after_first_word(line).to_string());
        } else if parts[0] == "fires-in-turn"
            && parts.len() > 1
            && let Some(current) = pending.as_mut()
        {
            if current.fires_in_turn.is_some() {
                return Err(malformed(format!(
                    "{source} is malformed, the action {} carries two fires-in-turn lines",
                    current.name
                )));
            }
            current.fires_in_turn = Some(after_first_word(line).to_string());
        } else {
            return Err(malformed(format!(
                "{source} is malformed, the line {line} matches no catalogue line form"
            )));
        }
    }
    if let Some(done_pending) = pending.take() {
        let done = finish_action(done_pending, source)?;
        actions.insert(done.name.clone(), done);
    }
    Ok(Catalogue {
        harness,
        facts,
        actions,
    })
}
