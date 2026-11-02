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
