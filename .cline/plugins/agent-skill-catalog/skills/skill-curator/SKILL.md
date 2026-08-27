---
name: skill-curator
description: This skill should be used when the user asks to "review external agent skills", "curate awesome-agent-skills", "select skills to package", "triage a skill catalog", "build a skill pack", or decide which third-party agent skills are worth adapting for AutoDev.
version: 0.1.0
---

# Skill Curator

Curate external agent skills as candidate metadata before any active installation. Treat catalogs such as `VoltAgent/awesome-agent-skills` as discovery indexes, not audited dependencies.

## Core workflow

1. Identify the user goal and target workflow category.
2. Build a candidate list from source URLs and catalog metadata.
3. Remove candidates that require prohibited authority, unknown credentials, production write access, or destructive operations.
4. Group remaining candidates by workflow value: planning, context, review, testing, deployment, documentation, MCP, security, observability, or domain-specific operations.
5. Record metadata only until license and security reviews pass.
6. Prefer original AutoDev-native adaptations over copied external skill bodies.
7. Promote a candidate only when review evidence is sufficient.

## Candidate scoring

Score each candidate on five axes:

- **Fit:** direct relevance to AutoDev workflows.
- **Authority:** whether it can remain advisory or requires sensitive capabilities.
- **Evidence:** docs, tests, examples, or clear provenance.
- **Maintainability:** stability of source and clarity of ownership.
- **Uniqueness:** whether it adds something not already covered by local skills.

Reject low-fit candidates even if popular. Prefer fewer high-quality skills over broad noisy imports.

## Metadata-only record

For each candidate, capture:

```json
{
  "id": "owner-repo-path-or-stable-slug",
  "source_url": "https://example.invalid/path/to/skill",
  "source_repo": "owner/repo",
  "source_revision": "commit-or-release-if-known",
  "status": "discovered",
  "category": "context-engineering",
  "vendoring_mode": "metadata-only",
  "license": "unknown",
  "risk": "unreviewed",
  "authority": {
    "filesystem": "none",
    "shell": "none",
    "network": "advisory-only",
    "secrets": "forbidden",
    "git": "none",
    "production": "none"
  },
  "notes": "Review required before activation."
}
```

## Promotion criteria

Promote a candidate from discovery to packaging only when:

- license review identifies a compatible use mode;
- security review finds no instruction conflicts or unsafe authority assumptions;
- the skill can operate inside AutoDev workspace, approval, and evidence boundaries;
- validation commands or manual review checks are named;
- attribution and source links are preserved.

## AutoDev boundaries

Keep curation advisory. Do not install MCP servers, execute downloaded scripts, change Cline settings, read secrets, or grant tool permissions as part of curation. Escalate only to explicit implementation tasks after review.

## Additional resources

- `references/curation-rubric.md` — scoring rubric and promotion states.
