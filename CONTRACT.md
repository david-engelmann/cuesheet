# Contract - cuesheet

This contract is the executable spec for cuesheet. Every obligation below is stated so it can fail. The full contract is the first build act of this repo and it extends the stub obligations written here.

## Stub obligations

CU-S1. Every cue on the sheet names the action, what fires it, and what it fires in turn, and a cue missing any of those is rejected rather than listed incomplete.

CU-S2. The sheet lists every action available at the decision point it was generated for, and an available action missing from the sheet is a defect the tests must catch.

CU-S3. The sheet lists no action that is not available at that decision point, and a cue for an unavailable action is a defect the tests must catch.

CU-S4. Cuesheet lists and never executes, and generating a sheet changes no state outside the sheet itself, because the sheet is a surface for a decision and not an actor in the run it describes.

CU-S5. Every fixture and example in this repo is synthetic and built for this repo, and no real trace or session data appears anywhere in the tree.

## Round 1 preamble

Round 1 builds the in-context action surface for one harness at one decision point. Two recorded inputs feed it. A catalogue fixes the set of actions one harness family can ever offer and declares every fact a context for that family can carry. A context records one harness at one decision point and states the facts the catalogue rules read. Both formats are line based and are defined in full here, and nothing outside these two files is read.

A catalogue opens with a header line naming the harness family. Fact declarations follow, each naming one fact and the values it can carry. A value fact carries exactly one of its declared values at a time. A set fact carries any subset of its declared tokens. Action entries follow, and each entry is a block that opens with the action name and then carries its rule lines and its two text lines in any order after the first. A rule line is one fact test, and the rule is the conjunction of its test lines. The grammar is this.

```
catalogue <harness>
fact <name> values <value>, <value>, ...
fact <name> set-of <token>, <token>, ...
action <name>
requires <fact> is <value>
requires <fact> is-not <value>
requires <fact> has <token>
requires <fact> lacks <token>
fires <text>
fires-in-turn <text>
```

The tests named is and is-not read value facts. The tests named has and lacks read set facts. A context opens with a header line naming the harness and the decision point, and then carries one line per recorded fact. The grammar is this.

```
context <harness> <decision-point>
fact <name> = <value>
fact <name> = <token>, <token>, ...
```

A context may record any subset of the facts the catalogue declares, and a fact it does not record is missing for that context. The sheet for one context lists cues sorted by action name, and each cue carries the action name, the fires text, the fires-in-turn text, and one evidence line per rule test in catalogue order. An evidence line names the fact, the test, the tested value, and the recorded value it was checked against, with a recorded set printed sorted.

## Round 1 obligations

CU-1. A context that is malformed is refused with the defect named, and the refusal covers a missing header line, a header that names no decision point, a fact line that matches no declared fact, a fact recorded twice, a recorded value outside the declared values for that fact, a recorded token outside the declared tokens for that fact, and a line that matches no context line form, and no refusal ever surfaces as a traceback or as a partial sheet.

CU-2. A catalogue declares each fact once with its kind and its full value list, and each action once with at least one rule test and exactly one fires line and one fires-in-turn line, and the catalogue text is the only source of facts and actions the sheet can ever draw on.

CU-3. A catalogue that is malformed is refused at load with the defect named, and the refusal covers a missing header line, a fact declared twice, an action declared twice, a rule test that names a fact the catalogue never declares, a rule test whose form does not match the kind of the fact it names, a tested value outside the declared values or tokens of that fact, an action with no rule test, an action missing its fires line or its fires-in-turn line, and a line that matches no catalogue line form.

CU-4. A context whose harness differs from the catalogue harness is refused with both harness names in the refusal, because a sheet is defined only over one catalogue and one context from the same harness family.

CU-5. An action is listed on the sheet when every rule test in its catalogue entry passes against the facts the context records, and an action is absent when any one of its rule tests fails, because the rule is a conjunction and no subset of passing tests is enough.

CU-6. A rule test that names a fact the context does not record makes its action unavailable, and the missing fact is never assumed, defaulted, or read from any other source, and completeness in CU-S2 is judged over the facts the context records and no others.

CU-7. Every cue on the sheet names the action, the fires text, the fires-in-turn text, and one evidence line per rule test in the order the catalogue entry states them, and each evidence line names the fact, the test, the tested value, and the recorded value the test was checked against.

CU-8. The sheet lists cues sorted by action name, no action appears on one sheet more than once, and generating the sheet twice over the same catalogue and context produces byte identical output, and the order of fact lines in the context and the order of action entries in the catalogue never change the sheet.

CU-9. When no action is available at the decision point the sheet is its header line alone, and an empty sheet is a produced sheet and not a refusal.

CU-10. The command line tool takes one catalogue path and one context path, exits with code 0 and prints the sheet when the sheet is produced, and exits with code 2 and prints one error line and no sheet when either input is malformed or cannot be read, and a refusal never surfaces as a traceback.

CU-11. Generating a sheet reads only the catalogue and context files it is handed and changes no state anywhere, and the bytes of both input files are the same after the sheet is generated as before.

CU-12. The examples in this repo are one synthetic catalogue for one harness, two recorded contexts for that harness, and the two sheets those contexts produce, and the committed sheets are byte identical to what the tool produces from the committed catalogue and contexts.

CU-13. Amendment, added by the Round 1 review. The preamble's phrase for a set fact, that it carries any subset of its declared tokens, is read with the context grammar, which records one or more tokens on a fact line. A set that was observed to hold nothing is therefore recorded by leaving the fact out of the context, and CU-6 governs it, so a rule test naming that fact does not pass and an action gated only on such a test is unavailable in that context. No separate empty recording form exists.

## Round 2 preamble

Round 2 adds the companion view to the sheet. The sheet names what can fire and stays silent about the rest, so an auditor checking the sheet against CU-S3 has to recompute the catalogue by hand. The gaps view answers for the other side. For the same catalogue and the same context it names every action the sheet leaves off and states which of its rule tests did not pass. Both views come out of one evaluation of the rule tests, so each action is judged once and the two views divide the catalogue between them. The gaps view is a surface like the sheet, and it executes nothing and changes no verdict the sheet makes.

The printed form of the gaps view is this. A header line names the harness and the decision point, as the sheet header does. One block follows per unavailable action, sorted by action name. A block opens with a line naming the action and then carries one blocked line per rule test that did not pass, in the order the catalogue entry states its tests. A blocked line names the fact, the test, and the tested value, and then the recorded value the test was checked against, or the words not recorded when the context records no value for that fact. The grammar is this.

```
gaps <harness> <decision-point>
gap <action>
blocked <fact> <test> <value>, recorded <recorded value>
blocked <fact> <test> <value>, not recorded
```

## Round 2 obligations

CU-14. A gaps subcommand takes the same catalogue path and the same context path as the sheet subcommand and prints the gaps view for that pair, and the gaps view carries one block for each action the catalogue declares that earns no cue on the sheet for that context, and the blocks stand sorted by action name as the sheet sorts its cues, and every action the catalogue declares appears on exactly one of the two views for a given context and never on both and never on neither, and both views are computed from one shared evaluation of the rule tests, so the two views cannot disagree about an action.

CU-15. A gap block opens with a line that names the action and then carries one blocked line for each rule test of that action that did not pass, in the order the catalogue entry states its tests, and each blocked line names the fact, the test, and the tested value, and then the recorded value the test was checked against when the context records that fact, and the words not recorded when the context records no value for that fact, and a rule test that names a fact the context does not record never passes and always takes the not recorded form, because the missing fact is never assumed under CU-6, and a test that passed is never printed in a gap block, because the block states why the action is unavailable and nothing else, and every gap block carries at least one blocked line, because an action earns a gap block only when at least one of its tests did not pass and CU-3 already refuses an action with no rule test at load.

CU-16. The gaps subcommand exits with code 0 whenever it prints the gaps view, and when no action is unavailable the gaps view is its header line alone and still exits with code 0, as the empty sheet does under CU-9, and a malformed catalogue or a malformed context or a file that cannot be read refuses under the gaps subcommand with the same error line and the same exit code 2 the sheet subcommand gives for the same inputs, and a refusal never surfaces as a traceback and never prints a partial gaps view, and the output of the sheet subcommand is byte identical after this round to what it was before it, for every catalogue and context.

CU-17. The examples in this repo gain the two gaps views for the recorded contexts, committed beside the two sheets in the examples directory under the names ordinary.gaps and restricted.gaps, and the committed gaps views are byte identical to what the tool produces from the committed catalogue and contexts, and the restricted gaps view names ask-human with its human-present test answered in the not recorded form.

CU-18. Amendment, added by the Round 2 review. CU-14's shared evaluation sentence and the Round 2 preamble's phrase for it are restated to the strength the implementation supports. The two views are computed from one shared evaluation routine, and each action's rule tests are evaluated by that one routine for whichever view is being built, and because the routine is a pure function of the catalogue and the context handed to it, the two views cannot disagree about an action for a given catalogue and context. The preamble's wording that each action is judged once reads subject to this amendment. No behaviour changes under this amendment.

## Round 3 preamble

Round 3 settles how this format reads its own lists. Probes run against the Round 2 closed tree before this round showed that the parser already absorbs a repeated item in every list the two formats carry. A repeated option in the values list or the set-of list of a fact declaration, and a repeated token in the token list of a recorded set, each parse and produce a sheet and a gaps view byte identical to the item written once, and whitespace around a comma is never part of an option or a token. None of that stood written in this contract, and the red teams of the two earlier rounds had noted the set token half of it as behaviour without a stated reading. The same probes showed one place where the format does not read as a set. A rule test repeated inside one action is stored twice by the parser, and the sheet prints its evidence line twice, and a probe of the gaps view run before this round showed the matching defect on the other side, where a repeated failing test prints its blocked line twice. Two catalogues that state the same rule therefore produce different sheets today. This round states the list readings as they already stand and brings the rules to the same reading, so every list in the format and every rule in a catalogue is read as a set.

## Round 3 obligations

CU-19. The option list of a fact declaration, in the values form and in the set-of form alike, and the token list of a recorded set in a context are each read as a set, and an option or a token repeated inside one such list is the same declaration or the same recording as that item written once, and the sheet and the gaps view over an input that repeats an item in any of these lists are byte identical to the sheet and the gaps view over the same input with the item written once, and whitespace around a comma is no part of an option or of a token, and an empty option in a fact declaration and an empty token in a recorded set are refused as they were before this round, naming the fact whose list carries the empty item, because the set reading absorbs a repetition of a named item and never an unnamed one. A rule is likewise read as a set of its tests in first occurrence order, and a rule test repeated inside one action is the same rule as that test written once, and the parser records each distinct test of an action once, in the order the action first states it, and the evidence line of a repeated passing test prints once on the sheet and the blocked line of a repeated failing test prints once in the gaps view, and a test stated by two different actions is kept by each of them, because the set reading of a rule holds inside one action and never across actions.

## Round 4 preamble

Round 4 ports the landed behaviour of Rounds 1 to 3 to Rust. The Python implementation at the Round 3 closure is the oracle, and the port reproduces its functional outcomes exactly, meaning exit codes, stream contents, and file effects, over the committed examples and over generated inputs. Help text and malformed invocation error text follow clap rather than the oracle, and nothing else about the surface changes. Four readings that a first pass over the Python code suggests were probed against the running oracle before this round was written, and each probe corrected the reading. The file a read opens is the pathlib normalized spelling of the argument and not the argument itself, so a trailing slash on a regular file reads fine, while the cannot be read line names the argument exactly as written. A separator token is struck as a separator only the first time one appears, and every later separator is an ordinary value, and the distribution can hand a positional a list of values where the oracle then dies uncaught. A trailing separator after both paths assigns normally, and a separator standing in for a missing path never satisfies it. A write to a dead stream turns on how much text the interpreter has staged, with a document of at most 8191 bytes failing at exit code 120 in the shutdown flush and a document of 8192 bytes or more failing at exit code 1 inside the write itself, and the port answers every dead stream write in its own voice under CU-24 instead. The probes behind these readings are recorded in the port lane's probe notes and are pinned by the differential harnesses of CU-26.

## Round 4 obligations

CU-20. This round ports the behaviour stated by CU-1 to CU-19 to a Rust implementation, and the port obligations of this round begin at CU-20. A catalogue subcommand slice that was built in Python and never landed used CU-20 to CU-23 in its local numbering, and that numbering is superseded on this branch, and the catalogue work takes fresh numbers when it lands as a later Rust round. The Python implementation is removed in this round once parity is proven, and from this round the Rust crate is the implementation the contract describes, and no obligation from CU-1 to CU-19 changes its meaning under the port.

CU-21. For every catalogue and context pair and every argument shape the port is handed, the Rust binary gives the same exit code and the same bytes on the same stream as the oracle, and the refusals carry the oracle wording at every site, and the only text that differs is the help text and the malformed invocation error text that follow clap, and the crash family answers that CU-23 and CU-24 state in the voice of the port itself.

CU-22. The argument distribution follows the probed reading of the oracle parser. The first separator token is struck as a separator and every later separator token is an ordinary value, and the first separator inside the slice a positional receives is struck before the value is read, and a trailing separator after both paths leaves the assignment untouched, and a separator never stands in for a missing path, and an incomplete distribution refuses with the missing arguments named before any unrecognized argument is reported, and extra values and unknown flags after a complete invocation are reported at the top level parser, and help fires wherever a help token stands before the first separator, with the attached argument and cluster outcomes the probes recorded.

CU-23. Where the distribution hands a positional a list of values rather than one value, which the separator shapes of the probes produce for the context slot as an empty list, the oracle raises an uncaught exception and prints a traceback, and the port instead exits with code 1 and prints nothing on either stream.

CU-24. When a write to standard output or standard error fails because the stream is dead, the port exits with code 1 and writes nothing further, for a document, for help text, for a refusal line, and for usage text alike, and the exit code 120 shutdown artifact of the oracle and its interpreter notices and tracebacks on dead streams are not reproduced.

CU-25. Each read opens the normalized spelling of the path it was handed, where normalization drops empty components and dot components and a trailing slash and keeps parent components for the operating system to resolve and keeps a leading slash and an exactly double leading slash, and the cannot be read line names the path exactly as written, and a directory, an empty path, and a file whose bytes are not UTF-8 each refuse as a file that cannot be read, and the catalogue is read and judged before the context file is touched.

CU-26. The landed Python tests stand ported test for test as cargo tests, and the committed catalogue and contexts reproduce their committed sheets and gaps views byte for byte through the Rust binary, and the partition property of CU-14 is tested over generated catalogue and context pairs as well as the committed ones, and differential harnesses compare the Rust binary against the oracle over an argv corpus and a document corpus of committed examples and generated synthetic inputs, comparing exit code and both streams byte for byte, and the corpus file pins its per source case counts with the oracle cases and the clap presentation cases counted apart.

CU-27. The crate is one Rust crate on edition 2024 with a library target carrying the behaviour and a binary target carrying the command line, and its dependencies are clap 4 in derive form with long argument inference and thiserror 2 for library errors and anyhow 1 confined to the binary entry, and no crate of the serde family appears anywhere in the manifest, in the dependencies or the dev dependencies or the build dependencies or under any target table, and every non Rust file left in the tree is documentation or contract prose or a licence, the manifest itself, a continuous integration file that only invokes the cargo gate, or a declarative fixture or example the Rust tests read and assert, and the port report names each remainder and its justification.

CU-28. The setup invariants stand re-expressed as Rust tests, and they pin the canonical template hash of AGENTS.md, the absence of U+2014 and U+2013 from every text file in the tree, the closed round count in the README standing at 3 for this round, and a continuous integration file whose gate step runs the cargo test suite, and the local gate for this round is the cargo suite green in the worktree and again from a fresh extraction of the final tree, with formatting clean and clippy at zero warnings.

CU-29. Amendment, added by the Round 4 build probes. CU-24 is corrected on the strength of the dead stream probes taken against the running oracle while this round was being built. A write to a dead standard error stream never raises inside the oracle at any size, and text staged for a dead stream that cannot be flushed at shutdown turns the exit of the whole run to 120 whatever the run had otherwise earned, while a document or help text of 8192 bytes or more written to a dead standard output raises inside the write and earns 1 when the error stream can still take the report. The port therefore answers a failed error stream write with 120 at every size, and a failed document or help write under 8192 bytes with its own could not print line on the error stream and exit 120, and a failed document write at or past 8192 bytes with its own document print failed line on the error stream and exit 1, and a failed write of either of those lines answers 120 as well. CU-24 stands corrected to this reading, and its flat exit 1 no longer governs.

CU-30. Amendment, added by the Round 4 build probes. CU-23 is corrected in one detail. Where the distribution hands the context slot a list, the port prints its own line for the family on the error stream, naming that an argument arrived as a list where a single path was expected, and exits with code 1, and that line is written under the same discipline as every error line of the port, so a dead error stream turns the answer to 120. The list surfaces only when the catalogue slot holds one path and that catalogue has been read and has parsed clean, because the oracle reads in slot order and a catalogue refusal answers first, and the catalogue slot itself never holds a list in any distribution where the context slot stands assigned.

CU-31. Amendment, added by the Round 4 build. CU-27 is corrected in its dependency placement sentence. No crate of the serde family stands in the shipped dependencies or the build dependencies, and the shipped sources import no crate of that family. serde_json stands as a dev dependency only, where the corpus reader in the tests uses it, and sha2 stands as a dev dependency only, where the setup hash pin in the tests uses it, because neither corpus reading nor digest reading is product behaviour in this repo. The shipped dependency set CU-27 names stands unchanged.
