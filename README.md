# cuesheet

Lists the actions available to an agent at one decision point, in context, and never executes one.

Cuesheet reads a catalogue and a context. The catalogue declares every action one harness family can offer and the facts those actions turn on, and the context records the facts at one decision point. Cuesheet prints the actions available at that point with the evidence that fires each one, the actions that are not available with the tests that block them, or the catalogue itself in the recorded form the parser holds. It lists and never executes.

[![CI](https://github.com/david-engelmann/cuesheet/actions/workflows/ci.yml/badge.svg)](https://github.com/david-engelmann/cuesheet/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

```console
$ ./target/debug/cuesheet sheet examples/stagehand.catalogue examples/ordinary.context
sheet stagehand after-read
cue ask-human
fires a person is standing by and the decision is theirs to make
fires-in-turn an answer the run can act on
evidence human-present is yes, recorded yes
cue edit-file
fires the file in context is the one the decision just made
fires-in-turn a changed file the run can test or describe
evidence tools has edit, recorded edit, message, read, run, search
evidence last-action is read-file, recorded read-file
cue read-file
fires a file the run needs is not yet in context
fires-in-turn the file contents in context for the next decision
evidence tools has read, recorded edit, message, read, run, search
cue run-command
fires a check or build step is the next thing the decision needs
fires-in-turn the command output the run must react to
evidence tools has run, recorded edit, message, read, run, search
evidence usage-window is inside, recorded inside
cue search-corpus
fires the run needs evidence it has not read yet
fires-in-turn the passages the next decision can cite
evidence tools has search, recorded edit, message, read, run, search
cue send-update
fires the decision changed what someone waiting should know
fires-in-turn a message delivered to the person waiting
evidence tools has message, recorded edit, message, read, run, search
```

## Install

Cuesheet is a Rust crate that builds from source with cargo.

```sh
git clone https://github.com/david-engelmann/cuesheet.git
cd cuesheet
cargo build
```

The binary lands at target/debug/cuesheet, and the test suite runs with `cargo test`.

## Quickstart

From the repository root, with the committed examples.

```sh
./target/debug/cuesheet sheet examples/stagehand.catalogue examples/ordinary.context
./target/debug/cuesheet catalogue examples/stagehand.catalogue
```

The first command prints the sheet at the top of this page. The second prints the catalogue document in full, and its output stands in the Usage section below.

## Usage

### sheet

`cuesheet sheet <catalogue> <context>` prints the sheet for the decision point the context records, one cue per available action, sorted by action name, each cue carrying what the action fires, what it fires in turn, and one evidence line per rule test. Its output is the block at the top of this page.

### gaps

`cuesheet gaps <catalogue> <context>` prints the gaps view for the same pair, one block per action the sheet leaves off, each block naming the rule tests that did not pass and what each was checked against.

```console
$ ./target/debug/cuesheet gaps examples/stagehand.catalogue examples/ordinary.context
gaps stagehand after-read
gap spawn-helper
blocked tools has spawn, recorded edit, message, read, run, search
```

### catalogue

`cuesheet catalogue <catalogue>` prints the catalogue document, the catalogue in the recorded form the parser holds, one line per declared fact and one block per declared action, each in declaration order, with one evidence line per recorded rule test.

```console
$ ./target/debug/cuesheet catalogue examples/stagehand.catalogue
catalogue stagehand
fact tools set-of read, edit, search, run, spawn, message
fact human-present values yes, no
fact usage-window values inside, outside
fact last-action values none, read-file, edit-file, search, run-command, ask-human, spawn-helper, send-update
action read-file
evidence tools has read
action edit-file
evidence tools has edit
evidence last-action is read-file
action search-corpus
evidence tools has search
action run-command
evidence tools has run
evidence usage-window is inside
action ask-human
evidence human-present is yes
action spawn-helper
evidence tools has spawn
evidence usage-window is inside
action send-update
evidence tools has message
```

Every command reads only the files it is handed, and a file that is malformed or cannot be read refuses with one error line and exit code 2. No command executes an action it lists.

## Status

Round 3 landed in PR 6 and is recorded closed here. The set reading of the lists and the rules of the format is in place under contract obligations CU-1 to CU-19, so a repeated option in a fact declaration, a repeated token in a recorded set, and a repeated rule test inside one action each read as the item written once, and the sheet and the gaps view read the same recorded catalogue and context as before. Round 4 landed in PR 8 and is recorded closed here. It ports the implementation to Rust under the same contract, removing the Python tree and making the cargo suite the primary gate, and it changes no behaviour the earlier rounds landed. Round 5 landed in PR 10 and is recorded closed here. It adds the catalogue surface under contract obligations CU-32 to CU-36, printing the catalogue itself in declaration order through the same read and parse path the sheet uses, and it rebuilds this README to the fleet README standard with captured output throughout. Round 6 landed in PR 14 and is recorded closed here. It closes the open notes of the Round 4 port review under contract obligations CU-37 to CU-40, adjudicating the hand to clap fallback by probe and naming it in the contract, stating the crash family's governing answers directly in place of CU-21's compressed phrase, and folding the two print failure error types into one kind carrying type. Round 7 landed in PR 16 and is recorded closed here. It pins the repo's one description at every position it stands on under contract obligations CU-41 to CU-43, the lede of this README, the package description field, the root command's about text, the crate doc opening, and the catalogue of help expectations the corpus carries. Round 8 landed in PR 18 and is recorded closed here. It deepens the committed corpus across the branches of the argument classification under contract obligations CU-44 to CU-47, every new expectation derived from the contract before it ran, with a census pin holding the branch floors. Round 9 landed in PR 20 and is recorded closed here. It pins the branches of the argument classification with direct unit tests under contract obligations CU-48 to CU-51, forty one pins over the published scan surface, one family per branch family of the classifier. Round 10 landed in PR 22 and is recorded closed here. It pins the classifier's boundary shapes and the extra value interaction under contract obligations CU-52 to CU-55, nine unit pins over the shapes the Round 9 review named and one corpus case standing behind the interaction. Round 11 landed in PR 24 and is recorded closed here. It fires the catalogue command behind its states in the committed corpus under contract obligations CU-56 to CU-59, twenty three oracle cases over the print branches and every parse refusal family of the catalogue reader. Round 12 landed in PR 26 and is recorded closed here. It fires the gaps command behind its states in the committed corpus under contract obligations CU-60 to CU-63, five oracle cases over the branches of the gaps computation the census found unfired. Round 13 landed in PR 28. It fires the context reader's refusal families behind the sheet and gaps commands in the committed corpus under contract obligations CU-64 to CU-67, nine oracle cases over the families the census found unfired. Round 14 landed in PR 30. It deepens the firing coverage of the catalogue reader behind the catalogue command under contract obligations CU-68 to CU-71, eighteen oracle cases over the sub-forms of its compound checks and the orders among its refusals. Round 15 landed in PR 32. It deepens the firing coverage of the context reader behind the sheet command and the catalogue reader behind the catalogue command under contract obligations CU-72 to CU-75, five oracle cases over the disjuncts of the context reader's compound checks and the length conjunct of the fires-in-turn branch. Round 16 landed in PR 34. It deepens the firing coverage of the catalogue reader behind the catalogue and sheet commands under contract obligations CU-76 to CU-79, six oracle cases over the reader's remaining sibling shapes. Round 17 landed in PR 36, firing the sheet evaluation's ordering rules and the agreement of its two views behind the sheet and gaps commands in six committed corpus cases. Round 18 landed in PR 38, firing the recorded set in the evaluation behind the sheet and gaps commands in six committed corpus cases. Round 19 landed in PR 40, firing the negative number classification at its edges behind the sheet and gaps commands in six committed corpus cases. Round 20 landed in PR 42, firing the path spelling rule behind every file read in six committed corpus cases. Round 21 landed in PR 44, firing the set repetition rule behind the sheet and gaps views in four committed corpus cases. Round 22 landed in PR 46, firing the context reader edges behind the sheet and gaps commands in six committed corpus cases. Round 23 landed in PR 48, firing the catalogue document and evaluation view shapes behind the sheet, gaps, and catalogue commands in six committed corpus cases. Round 24 landed in PR 50, firing the gaps view blocked shapes behind the gaps command in six committed corpus cases. Round 25 landed in PR 52, firing the repeated and contradictory test shapes behind the sheet and gaps commands in six committed corpus cases. Round 26 landed in PR 54, pinning the negative number matcher edges behind the sheet command in six committed corpus cases. Round 27 landed in PR 56, firing the context reader order behind the sheet command in six committed corpus cases. Round 28 landed in PR 58, firing the catalogue reader branch order behind the catalogue command in six committed corpus cases. Round 29 landed in PR 60, firing the sheet and gaps read order behind the sheet and gaps commands in six committed corpus cases. Round 30 landed in PR 62, pinning the argument scan walk order behind every command in six committed corpus cases. Round 31 landed in PR 66, pinning the Python whitespace line discipline behind every command in six committed corpus cases. Round 32 landed in PR 70, pinning the case discipline of the two readers and the evaluation in six committed corpus cases. Round 33 landed in PR 74, pinning the whitespace discipline of values and tokens in the two readers in six committed corpus cases. Round 34 landed in PR 77, pinning the name grammar of the two readers in six committed corpus cases. Round 35 landed in PR 82, pinning the fires text whitespace runs in six committed corpus cases. Round 36 landed in PR 86, pinning the sheet head discipline in six committed corpus cases. Round 37 landed in PR 90, pinning the byte order mark at the head of a body line in six committed corpus cases. Round 38 landed in PR 93. Round 39 landed in PR 97. Closed rounds 39 of 100. Part of the impatient programming portfolio, in the Fleet Operations arc.

## License

MIT. See LICENSE.
