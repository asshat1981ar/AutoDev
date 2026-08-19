# Repair direction

Repair consumes concrete verifier findings and previous attempt evidence. It should change the minimum necessary surface, preserve successful evidence where still valid, and re-run the checks invalidated by the repair.

Repeated repairs that reproduce the same failure should trigger diagnosis/replan rather than consuming the full budget with equivalent edits.
