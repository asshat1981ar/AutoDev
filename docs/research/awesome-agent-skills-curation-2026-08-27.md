# Awesome Agent Skills curation notes — 2026-08-27

Source: https://github.com/VoltAgent/awesome-agent-skills

Reviewed revision: not pinned in this metadata-only pass. Pin an exact commit before vendoring any linked skill content.

The upstream repository describes itself as a curated collection of 1000+ agent skills from official teams and the community, compatible with multiple AI coding assistants. It is useful as a discovery index, not as a blindly trusted source of executable agent behavior.

## Decision

Create an AutoDev-native metadata-only catalog plugin instead of bulk-vendoring the external skill corpus.

## Rationale

- Each linked skill can have separate licensing and attribution requirements.
- Skills are behavioral instructions and may weaken local safety rules if installed without review.
- Some skills assume direct shell, Git, MCP, browser, cloud, payment, or deployment authority that conflicts with AutoDev's ForgeCore boundary.
- Bulk-installed skills create selection noise and context bloat.

## Initial packaging scope

Package only original review workflows:

- skill curation;
- skill security auditing;
- skill license review;
- skill packaging.

## Excluded from initial scope

- copied third-party skill bodies;
- external scripts;
- MCP server registrations;
- credential-requiring skills;
- production/cloud-admin skills;
- destructive-operation skills.

## Promotion gate

A candidate external skill may become active only after:

1. source and revision are recorded;
2. license is identified;
3. copied/adapted/linked content mode is declared;
4. authority requirements are classified;
5. prompt-injection and policy-conflict review passes;
6. tests or validation commands are defined;
7. attribution is included.
