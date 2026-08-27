# License decision template

```markdown
## License review

Candidate: <id>
Source URL: <url>
Source repository: <owner/repo>
Revision: <rev>
Source paths reviewed:
- <path>
Source license: <SPDX or unknown>
Vendoring mode: metadata-only | linked-reference | adapted-summary | copied-text | copied-script | bundled-asset
Security review result: pass | conditional pass | reject
Authority classification: none | workspace-read | workspace-write | shell-approval | network-approval | secret-denied | production-denied | destructive-denied
Open conditions:
- <condition or none>
Decision: approved | approved-with-conditions | not-approved
Attribution required: yes | no | unknown
Notice required: yes | no | unknown
Notes:
- ...
```

Promotion rule: only `Decision: approved` with no open conditions, final security `pass`, complete source/revision/path evidence, and authority classification may become an active skill.
