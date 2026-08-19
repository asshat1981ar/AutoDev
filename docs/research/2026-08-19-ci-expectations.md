# ExecPlan CI expectations

Focused workflow must pass:

```text
python scripts/check_execplan_contract.py
python -m unittest tests.test_execplan_contract -v
cargo fmt --all -- --check
cargo clippy -p forge-core --all-targets --all-features -- -D warnings
cargo test -p forge-core --test exec_plan
cargo test -p forge-core
```

The pull request's existing repository workflows remain required regression evidence. Any compile/format/clippy/test failure is treated as an implementation defect and corrected before final review.
