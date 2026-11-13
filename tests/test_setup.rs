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
        ["20"],
        "README status must report exactly one closed round count and it must be 9"
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
fn the_ci_gate_steps_run_the_full_gate() {
    let text = read(".github/workflows/ci.yml");
    assert!(
        text.contains("merge_group"),
        "the CI gate must run for the merge queue"
    );
    assert!(
        text.contains("cargo fmt --check"),
        "the CI gate must check formatting"
    );
    assert!(text.contains("cargo clippy"), "the CI gate must run clippy");
    assert!(text.contains("Run CI gate"), "the CI gate step is missing");
    assert!(
        text.contains("cargo test"),
        "the CI gate must run the cargo gate"
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

/// The one description of the repo, in the form the
/// manifest field and the about of the root command hold,
/// with no sentence period standing at its end (CU-41).
const DESCRIPTION: &str = "Lists the actions available to an agent at one decision point, in context, and never executes one";

/// The one description as a sentence, the form the README
/// lede and the crate doc opening hold (CU-41).
const DESCRIPTION_SENTENCE: &str = "Lists the actions available to an agent at one decision point, in context, and never executes one.";

#[test]
fn the_one_description_stands_at_its_positions() {
    // The lede of the README, compared as the whole line.
    let readme = read("README.md");
    let lede = readme.lines().nth(2).expect("README lede line");
    assert_eq!(
        lede, DESCRIPTION_SENTENCE,
        "the README lede is not the one description as a sentence"
    );

    // The description field of Cargo.toml, parsed as the
    // field of the package table and compared on the value
    // of the field.
    let manifest = read("Cargo.toml");
    let mut in_package = false;
    let mut fields = manifest.lines().filter(|line| {
        if line.starts_with('[') {
            in_package = *line == "[package]";
        }
        in_package && line.starts_with("description = ")
    });
    let field = fields.next().expect("manifest description field");
    assert!(
        fields.next().is_none(),
        "the manifest carries more than one description field"
    );
    let value = field
        .strip_prefix("description = \"")
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("manifest description value");
    assert_eq!(
        value, DESCRIPTION,
        "the manifest description is not the one description"
    );

    // The about attribute of the root command in src/main.rs,
    // read at its position and compared on its value.
    let main = read("src/main.rs");
    let mut abouts = main.lines().filter(|line| line.contains("about = "));
    let about_line = abouts.next().expect("root command about");
    assert!(
        abouts.next().is_none(),
        "src/main.rs carries more than one about attribute"
    );
    let about = about_line
        .split("about = \"")
        .nth(1)
        .and_then(|rest| rest.strip_suffix("\","))
        .expect("root command about value");
    assert_eq!(
        about, DESCRIPTION,
        "the about of the root command is not the one description"
    );

    // The crate doc opening in src/lib.rs, rebuilt from its
    // first two doc lines and compared whole.
    let lib = read("src/lib.rs");
    let mut doc_lines = lib.lines();
    let first = doc_lines
        .next()
        .and_then(|line| line.strip_prefix("//! "))
        .expect("crate doc first line");
    let second = doc_lines
        .next()
        .and_then(|line| line.strip_prefix("//! "))
        .expect("crate doc second line");
    let opening = format!("{first} {second}");
    assert_eq!(
        opening,
        format!("cuesheet, the library. {DESCRIPTION_SENTENCE}"),
        "the crate doc opening does not carry the one description"
    );

    // The root help expectations of the committed corpus,
    // counted and each compared on its first line. A stream
    // carries the root help text when it lists the commands
    // of the root command, which the error streams that print
    // the root usage line never do.
    let corpus = read("tests/cli_corpus.json");
    let parsed: serde_json::Value = serde_json::from_str(&corpus).expect("parse corpus");
    let cases = parsed
        .get("cases")
        .and_then(|cases| cases.as_array())
        .expect("corpus cases");
    let mut help_streams = Vec::new();
    for case in cases {
        let expect = case.get("expect").expect("expect block");
        for stream in ["stdout", "stderr"] {
            let text = expect
                .get(stream)
                .and_then(|text| text.as_str())
                .unwrap_or("");
            if text.contains("\nCommands:\n") {
                help_streams.push(text);
            }
        }
    }
    assert_eq!(
        help_streams.len(),
        10,
        "the corpus must carry exactly ten root help expectations"
    );
    for text in help_streams {
        let first_line = text.split('\n').next().expect("help first line");
        assert_eq!(
            first_line, DESCRIPTION,
            "a root help expectation does not open with the one description"
        );
    }
}
