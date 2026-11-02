# Contract - cuesheet

This contract is the executable spec for cuesheet. Every obligation below is stated so it can fail. The full contract is the first build act of this repo and it extends the stub obligations written here.

## Stub obligations

CU-S1. Every cue on the sheet names the action, what fires it, and what it fires in turn, and a cue missing any of those is rejected rather than listed incomplete.

CU-S2. The sheet lists every action available at the decision point it was generated for, and an available action missing from the sheet is a defect the tests must catch.

CU-S3. The sheet lists no action that is not available at that decision point, and a cue for an unavailable action is a defect the tests must catch.

CU-S4. Cuesheet lists and never executes, and generating a sheet changes no state outside the sheet itself, because the sheet is a surface for a decision and not an actor in the run it describes.

CU-S5. Every fixture and example in this repo is synthetic and built for this repo, and no real trace or session data appears anywhere in the tree.
