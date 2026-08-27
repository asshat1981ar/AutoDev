---
name: skill-license-reviewer
description: This skill should be used when the user asks to "license review a skill", "check whether a skill can be vendored", "add attribution for external skills", "package skills from a catalog", or decide whether third-party agent skill text, scripts, or examples may be copied into AutoDev.
version: 0.1.0
---

# Skill License Reviewer

Review licensing and attribution before copying or adapting external skill content. Do not assume an index repository license applies to every linked skill.

## Review workflow

1. Identify the exact source repository, file path, and revision.
2. Locate the source license file and any file-level notices.
3. Determine whether the intended use is metadata-only, linked-reference, adapted summary, copied text, copied script, or bundled asset.
4. Record source repository, exact source revision or release, source file paths, security review result, and AutoDev authority classification.
5. Check whether attribution, notice preservation, license text, or source modification disclosure is required.
6. Record unknown or missing license as `not-approved-for-vendoring`.
7. Prefer original AutoDev-native instructions with source links when licensing is uncertain.
8. Add attribution notes before promotion.

## Vendoring modes

- `metadata-only`: source link and review notes only. Lowest risk.
- `linked-reference`: link to external content without copying. Low risk but may require network.
- `adapted-summary`: original local summary inspired by reviewed concepts. Requires attribution when appropriate.
- `copied-text`: copied skill body or examples. Requires compatible license and notices.
- `copied-script`: executable copied code. Requires license, security, and validation review.
- `bundled-asset`: non-code asset. Requires explicit license review.

## Required manifest fields

Record license decisions in candidate metadata:

```json
{
  "source_url": "https://example.invalid/skill",
  "source_repository": "owner/repo",
  "source_revision": "commit-or-release",
  "source_paths": ["skills/example/SKILL.md"],
  "license_spdx": "MIT",
  "vendoring_mode": "metadata-only",
  "security_review_result": "pass",
  "authority_classification": "workspace-read",
  "attribution_required": true,
  "notice_file_required": false,
  "open_conditions": [],
  "approved_for_activation": false,
  "review_notes": "No third-party text copied."
}
```

Only records with complete source, revision, path, security, and authority evidence can be considered for activation. Conditional or incomplete license records are not promotable.

## Failure conditions

Reject active packaging when:

- no license can be found;
- license terms conflict with repository distribution;
- source combines incompatible copied snippets;
- attribution cannot be preserved;
- the requested vendoring mode is broader than the reviewed license permits.

## Additional resources

- `references/license-decision-template.md` — reusable decision note.
