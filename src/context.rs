//! A recorded harness context at one decision point.

use std::collections::{BTreeSet, HashMap};

use crate::catalogue::{Catalogue, VALUE};
use crate::errors::{CuesheetError, malformed};
use crate::lines::meaningful_lines;
use crate::pytext::{PyStr, py_split, py_strip};

/// One recorded value of a context: a value fact records one
/// of its declared values, and a set fact records a set of its
/// declared tokens, repetitions absorbed (CU-19).
#[derive(Debug, Clone, PartialEq)]
pub enum Recorded {
    Scalar(String),
    Tokens(BTreeSet<String>),
}

/// A parsed context.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub harness: String,
    pub decision_point: String,
    pub facts: HashMap<String, Recorded>,
}

/// Parse one context against a catalogue or refuse it.
pub fn parse_context(
    text: &str,
    source: &PyStr,
    catalogue: &Catalogue,
) -> Result<Context, CuesheetError> {
    let lines = meaningful_lines(text);
    if lines.is_empty() {
        return Err(malformed(format!(
            "{source} is malformed, it names no harness"
        )));
    }
    let head = py_split(lines[0]);
    if head.len() != 3 || head[0] != "context" {
        return Err(malformed(format!(
            "{source} is malformed, the first line does not name the harness and the decision point"
        )));
    }
    let harness = head[1].to_string();
    let decision_point = head[2].to_string();
    if harness != catalogue.harness {
        return Err(malformed(format!(
            "{source} names the harness {harness} and the catalogue names {}, and a sheet is defined only over one harness family",
            catalogue.harness
        )));
    }
    let mut recorded: HashMap<String, Recorded> = HashMap::new();
    for line in &lines[1..] {
        let parts = py_split(line);
        if parts.len() < 4 || parts[0] != "fact" || parts[2] != "=" {
            return Err(malformed(format!(
                "{source} is malformed, the line {line} matches no context line form"
            )));
        }
        let name = parts[1];
        let Some(fact) = catalogue.facts.get(name) else {
            return Err(malformed(format!(
                "{source} is malformed, the fact {name} is one the catalogue never declares"
            )));
        };
        if recorded.contains_key(name) {
            return Err(malformed(format!(
                "{source} is malformed, the fact {name} is recorded twice"
            )));
        }
        let raw = py_strip(
            line.split_once('=')
                .map(|pair| pair.1)
                .expect("the line form carries ="),
        );
        if fact.kind == VALUE {
            if !fact.options.iter().any(|option| option == raw) {
                return Err(malformed(format!(
                    "{source} is malformed, the fact {name} is recorded as {raw}, which the catalogue never declares for it"
                )));
            }
            recorded.insert(name.to_string(), Recorded::Scalar(raw.to_string()));
        } else {
            let tokens: Vec<&str> = raw.split(',').map(py_strip).collect();
            if tokens.iter().any(|token| token.is_empty()) {
                return Err(malformed(format!(
                    "{source} is malformed, the fact {name} carries an empty token"
                )));
            }
            for token in &tokens {
                if !fact.options.iter().any(|option| option == token) {
                    return Err(malformed(format!(
                        "{source} is malformed, the fact {name} carries the token {token}, which the catalogue never declares for it"
                    )));
                }
            }
            recorded.insert(
                name.to_string(),
                Recorded::Tokens(tokens.iter().map(|token| token.to_string()).collect()),
            );
        }
    }
    Ok(Context {
        harness,
        decision_point,
        facts: recorded,
    })
}
