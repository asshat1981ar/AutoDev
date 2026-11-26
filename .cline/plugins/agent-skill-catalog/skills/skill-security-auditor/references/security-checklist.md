# Skill security checklist

## Required evidence

- Source URL and revision recorded.
- All included files listed.
- Scripts reviewed or excluded.
- Required permissions classified.
- Conflicts with local harness rules identified.
- Secrets and production access denied by default.
- Validation steps documented.

## Decision template

```markdown
## Security review

Candidate: <id>
Source: <url>
Revision: <rev>
Decision: pass | conditional-pass | reject
Authority: <classification>
Findings:
- ...
Required changes before activation:
- ...
Evidence:
- ...
```
