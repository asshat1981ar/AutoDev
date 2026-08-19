# Security/adversarial test backlog

- Serialized imported harness containing `approved: true` cannot produce trusted approval.
- Plugin requesting broader capabilities than its declared manifest fails closed.
- Harness profile attempting to exclude policy/verification kernel behavior has no effect on the kernel.
- Malformed/unknown required evidence cannot complete a run.
- Symlink/path traversal through imported filesystem configuration remains workspace-confined.
- Sensitive environment requirements are represented as references/requirements, not persisted secret values in plan prose.
- Changed plugin integrity between checkpoint/resume triggers revalidation.
- Stale approval cannot be reused for a materially different effect.
