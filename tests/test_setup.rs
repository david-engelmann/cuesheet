//! The setup invariants, re-expressed as cargo tests from
//! the landed Python suite, with the port round's CI gate
//! invariant standing beside them (CU-28).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cuesheet::ndtable::{ND_RANGES, is_decimal_digit};
use sha2::{Digest, Sha256};

const CANONICAL_PREFIX_SHA256: &str =
    "63d2f04005dc8eb111d3efca1be55c0911f9ad46bc7cf3c74d9f9908f4a86f11";

const EXPECTED_OBLIGATIONS: [&str; 5] = [
    "CU-S1. Every cue on the sheet names the action, what fires it, and what it fires in turn, and a cue missing any of those is rejected rather than listed incomplete.",
    "CU-S2. The sheet lists every action available at the decision point it was generated for, and an available action missing from the sheet is a defect the tests must catch.",
    "CU-S3. The sheet lists no action that is not available at that decision point, and a cue for an unavailable action is a defect the tests must catch.",
    "CU-S4. Cuesheet lists and never executes, and generating a sheet changes no state outside the sheet itself, because the sheet is a surface for a decision and not an actor in the run it describes.",
    "CU-S5. Every fixture and example in this repo is synthetic and built for this repo, and no real trace or session data appears anywhere in the tree.",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).expect("read file")
}

fn decimal_value(point: u32) -> Option<u32> {
    ND_RANGES
        .iter()
        .find(|&&(lo, hi)| lo <= point && point <= hi)
        .map(|&(lo, _)| (point - lo) % 10)
}

/// One stub obligation line, read as the Python pin read it.
/// Its digit is in the Unicode decimal class, as the oracle
/// pattern's digit class was.
fn stub_obligation(line: &str) -> Option<(String, u32, String)> {
    let chars: Vec<char> = line.chars().take(6).collect();
    if chars.len() < 6
        || !chars[0].is_ascii_uppercase()
        || !chars[1].is_ascii_uppercase()
        || chars[2] != '-'
        || chars[3] != 'S'
        || chars[5] != '.'
    {
        return None;
    }
    let number = decimal_value(chars[4] as u32)?;
    Some((chars[..2].iter().collect(), number, line.to_string()))
}

#[test]
fn the_contract_stub_obligations_exist() {
    let text = read("CONTRACT.md");
    let mut found = Vec::new();
    for line in text.split('\n') {
        if let Some(obligation) = stub_obligation(line) {
            found.push(obligation);
        }
    }
    let texts: Vec<&str> = found.iter().map(|(_, _, line)| line.as_str()).collect();
    assert_eq!(
        texts, EXPECTED_OBLIGATIONS,
        "stub obligation text has changed without a contract amendment"
    );
    let numbers: Vec<u32> = found.iter().map(|(_, number, _)| *number).collect();
    assert_eq!(
        numbers,
        [1, 2, 3, 4, 5],
        "stub obligations must be exactly S1 to S5"
    );
    for (prefix, _, line) in &found {
        assert_eq!(prefix, "CU", "obligation is not a CU obligation");
        let words = line.split_whitespace().count();
        assert!(words >= 20, "obligation too thin at {words} words");
    }
}

#[test]
fn the_agents_canonical_block_is_present() {
    let text = read("AGENTS.md");
    assert!(text.contains("repo annex below this line"));
    let prefix = text.split("## Repo annex").next().expect("prefix");
    let digest = format!("{:x}", Sha256::digest(prefix.as_bytes()));
    assert_eq!(
        digest, CANONICAL_PREFIX_SHA256,
        "canonical block above the annex line has drifted"
    );
    let annex = text.split("## Repo annex").nth(1).expect("annex");
    assert!(annex.contains("CONTRACT.md"));
}

#[test]
fn the_readme_status_reports_the_round_count() {
    let text = read("README.md");
    assert_eq!(
        round_counts(&text),
        ["5"],
        "README status must report exactly one closed round count and it must be 5"
    );
}

/// Every count in a "Closed rounds N of 100" phrase whose
/// closing 100 is not followed by another decimal digit. This
/// is the findall pin of the Python form, written without a
/// regex engine and reading digits in the Unicode decimal
/// class as the oracle pattern did.
fn round_counts(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("Closed rounds ") {
        let after = &rest[start + "Closed rounds ".len()..];
        let digits: String = after
            .chars()
            .take_while(|c| is_decimal_digit(*c as u32))
            .collect();
        if !digits.is_empty() {
            let tail = &after[digits.len()..];
            if let Some(tail) = tail.strip_prefix(" of 100") {
                let followed = tail
                    .chars()
                    .next()
                    .is_some_and(|c| is_decimal_digit(c as u32));
                if !followed {
                    found.push(digits.clone());
                }
            }
        }
        rest = &rest[start + 1..];
    }
    found
}

#[test]
fn the_setup_digit_class_is_the_unicode_decimal_class() {
    assert_eq!(round_counts("Closed rounds \u{664} of 100"), ["\u{664}"]);
    assert!(round_counts("Closed rounds 3 of 100\u{664}").is_empty());
    let arabic = "CU-S\u{664}. Every cue on the sheet names the action.";
    assert_eq!(stub_obligation(arabic).map(|(_, n, _)| n), Some(4));
    let roman = "CU-S\u{2163}. Every cue on the sheet names the action.";
    assert_eq!(stub_obligation(roman), None);
}

#[test]
fn the_ci_backstop_step_runs_the_cargo_gate() {
    let text = read(".github/workflows/ci.yml");
    assert!(
        text.contains("Run CI backstop"),
        "the CI backstop step is missing"
    );
    assert!(
        text.contains("cargo test"),
        "the CI backstop must run the cargo gate"
    );
    assert!(
        text.contains("dtolnay/rust-toolchain"),
        "the CI gate needs the Rust toolchain step"
    );
    assert!(
        !text.contains("pytest"),
        "the Python gate is retired in this slice"
    );
}

const TEXT_SUFFIXES: [&str; 9] = [
    ".md", ".yml", ".yaml", ".rs", ".toml", ".txt", ".json", ".jsonl", ".lock",
];
const ANY_DEPTH_SKIP: [&str; 4] = [".git", "target", ".pytest_cache", "__pycache__"];
const ROOT_SKIP: [&str; 4] = [".venv", "venv", "node_modules", ".tox"];

fn collect_scanned(dir: &Path, depth: usize, out: &mut BTreeSet<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let file_type = entry.file_type().expect("file type");
        if file_type.is_dir() {
            if ANY_DEPTH_SKIP.contains(&name.as_str()) {
                continue;
            }
            if depth == 0 && ROOT_SKIP.contains(&name.as_str()) {
                continue;
            }
            collect_scanned(&path, depth + 1, out);
        } else if file_type.is_file() {
            let scanned = match path.extension().and_then(|e| e.to_str()) {
                None => true,
                Some(ext) => TEXT_SUFFIXES.contains(&format!(".{ext}").as_str()),
            };
            if scanned {
                out.insert(path);
            }
        }
    }
}

#[test]
fn no_em_or_en_dash_in_text_files() {
    let root = root();
    let mut scanned = BTreeSet::new();
    collect_scanned(&root, 0, &mut scanned);
    for anchor in [
        "CONTRACT.md",
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "src/main.rs",
    ] {
        assert!(
            scanned.contains(&root.join(anchor)),
            "anchor file {anchor} was not scanned"
        );
    }
    let em = "\u{2014}".as_bytes();
    let en = "\u{2013}".as_bytes();
    let mut bad = Vec::new();
    for path in &scanned {
        let bytes = std::fs::read(path).expect("read scanned file");
        if bytes.windows(3).any(|w| w == em || w == en) {
            bad.push(
                path.strip_prefix(&root)
                    .expect("under root")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    assert!(bad.is_empty(), "em or en dash found in {bad:?}");
}
