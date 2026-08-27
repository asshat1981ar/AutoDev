---
name: skill-packager
description: This skill should be used when the user asks to "package a reviewed skill", "convert a skill into a Cline plugin", "create an AutoDev skill from reviewed external material", "promote a quarantined skill", or add skill files after curation, license review, and security review have passed.
version: 0.1.0
---

# Skill Packager

Package reviewed skill candidates into AutoDev-compatible plugin skills. Require curation, license, and security reviews before promotion from metadata to active skill files.

## Packaging workflow

1. Confirm candidate status is `adaptation-ready` or equivalent.
2. Read the curation, license, and security review notes.
3. Require trusted host approval and final passing review records before writing active files.
4. Choose the safest implementation mode: original AutoDev-native guidance, adapted summary, or copied content only when approved.
5. Create a lean `SKILL.md` with YAML frontmatter containing `name`, `description`, and `version`.
6. Use third-person trigger descriptions with concrete phrases.
7. Keep the body imperative and concise.
8. Move detailed checklists and examples into `references/` or `examples/`.
9. Avoid scripts unless deterministic execution is necessary and reviewed.
10. Preserve source attribution in plugin `LICENSES.md`, skill references, or manifest records.
11. Run validation commands and inspect git status for unrelated drift.

Do not promote conditional, remediated-only, or draft evidence. Package only candidates with final `pass` security review, final license approval, authority classification, source revision, and explicit host approval.

## File layout

Use this structure for a normal packaged skill:

```text
.cline/plugins/<plugin-name>/skills/<skill-name>/
├── SKILL.md
└── references/
    └── <focused-guide>.md
```

Add `examples/` only for concrete, tested examples. Add `scripts/` only when the script is reviewed, deterministic, and safer than repeated generated code.

## Description pattern

Use frontmatter similar to:

```yaml
---
name: reviewed-skill-name
description: This skill should be used when the user asks to "specific task", "specific workflow", or "specific troubleshooting phrase".
version: 0.1.0
---
```

## Validation checklist

Before completion, verify:

- JSON manifests parse;
- every active skill has frontmatter with `name` and `description`;
- active skill bodies do not contain unreviewed copied third-party text;
- quarantine contains metadata only;
- no generated artifacts or unrelated files are included;
- repository-required tests or harness checks pass.

## Additional resources

- `references/packaging-checklist.md` — final promotion checklist.
