# Packaging checklist

## Promotion gate

- [ ] Candidate source URL recorded.
- [ ] Source revision recorded when available.
- [ ] License review passed for intended use mode.
- [ ] Security review passed or required changes applied.
- [ ] Authority classification is compatible with AutoDev.
- [ ] Attribution added.
- [ ] `SKILL.md` frontmatter includes `name`, `description`, and `version`.
- [ ] Description has concrete trigger phrases.
- [ ] References are linked from `SKILL.md`.
- [ ] Validation commands pass.

## Suggested validation commands

```bash
python -m py_compile install.py bootstrap_cline_mcp.py .cline/hooks/*.py .cline/plugins/project-fabric/tools.py
python -m unittest discover -s tests -v
python scripts/check_harness_drift.py --verbose
git diff --check
```
