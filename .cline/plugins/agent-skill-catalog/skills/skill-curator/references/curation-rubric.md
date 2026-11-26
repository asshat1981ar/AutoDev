# Curation rubric

## Status values

- `discovered`: candidate found, no review yet.
- `license-review`: source and license under review.
- `security-review`: authority and instruction safety under review.
- `adaptation-ready`: eligible for AutoDev-native adaptation.
- `packaged`: active skill created and validated.
- `rejected`: excluded with reason.

## Risk indicators

Reject or quarantine candidates that:

- ask for credentials or secret files;
- instruct direct shell execution without approval;
- bypass local policy or verification gates;
- assume cloud-admin, payment, deployment, or Git write authority;
- contain remote code execution, curl-pipe-shell, or unpinned downloads;
- lack a visible source, owner, or license;
- duplicate existing local skills without improvement.
