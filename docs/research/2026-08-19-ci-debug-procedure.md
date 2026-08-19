# CI debug procedure

If focused CI fails:

- format failure: inspect rustfmt diff/line and apply canonical formatting only;
- compile/clippy failure: inspect exact diagnostic and smallest affected API/test;
- focused test failure: classify implementation defect versus incorrect test/spec assumption; spec wins;
- Python contract failure: compare exact required PLANS.md fragments;
- unrelated repository CI failure: inspect logs and compare with main/pre-existing evidence before changing scope.

Every correction re-runs the covering job before completion is claimed.
