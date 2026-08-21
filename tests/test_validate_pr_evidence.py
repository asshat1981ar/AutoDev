import json
import tempfile
import unittest
from pathlib import Path

from scripts.validate_pr_evidence import validate_event


VALID_EVIDENCE = (
    "CI run / artifact / output: local verifier output: pull-request evidence: PASS"
)

VALID_BODY = f"""## Summary

Security guardrail update.

## Trust-boundary impact

- [x] No trusted execution or authorization changes
- [ ] Trusted-boundary impact present (check all applicable categories below)

## Verification

- [x] bash tests/test_ast_grep_rules.sh

### Commands run

bash tests/test_ast_grep_rules.sh
python -m unittest tests.test_validate_pr_evidence

### Evidence

{VALID_EVIDENCE}
"""


class PullRequestEvidenceValidationTests(unittest.TestCase):
    """Exercise visible verification-evidence requirements for submitted PR bodies."""

    def event_path(self, body: str) -> Path:
        """Write a temporary pull_request event containing the supplied body."""
        handle = tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False)
        json.dump({"pull_request": {"body": body}}, handle)
        handle.close()
        self.addCleanup(Path(handle.name).unlink, missing_ok=True)
        return Path(handle.name)

    def test_accepts_complete_submitted_evidence(self):
        """Accept a visible trust declaration, commands, and deterministic evidence."""
        self.assertEqual(validate_event(self.event_path(VALID_BODY)), [])

    def test_rejects_empty_commands_section(self):
        """Reject a Commands run section containing only the template comment."""
        body = VALID_BODY.replace(
            "bash tests/test_ast_grep_rules.sh\npython -m unittest tests.test_validate_pr_evidence",
            "<!-- Paste the exact verification commands executed for this change, one per line. -->",
        )
        errors = validate_event(self.event_path(body))
        self.assertTrue(any("Commands run" in error for error in errors), errors)

    def test_rejects_placeholder_evidence(self):
        """Reject an Evidence section whose only value is hidden placeholder text."""
        body = VALID_BODY.replace(
            VALID_EVIDENCE,
            "CI run / artifact / output:\n<!-- Link the exact-head CI run, artifact, or concise command output that supports the checked claims. -->",
        )
        errors = validate_event(self.event_path(body))
        self.assertTrue(any("Evidence" in error for error in errors), errors)

    def test_rejects_conflicting_top_level_trust_declarations(self):
        """Reject simultaneous no-impact and impact-present declarations."""
        body = VALID_BODY.replace(
            "- [ ] Trusted-boundary impact present",
            "- [x] Trusted-boundary impact present",
        )
        errors = validate_event(self.event_path(body))
        self.assertTrue(any("exactly one" in error for error in errors), errors)

    def test_rejects_complete_evidence_hidden_in_html_comment(self):
        """Reject a complete-looking evidence block when it is entirely invisible."""
        hidden_body = f"""## Summary

Invisible validation content.

<!--
## Trust-boundary impact

- [x] No trusted execution or authorization changes
- [ ] Trusted-boundary impact present (check all applicable categories below)

### Commands run

bash tests/test_ast_grep_rules.sh

### Evidence

{VALID_EVIDENCE}
-->
"""
        errors = validate_event(self.event_path(hidden_body))
        self.assertTrue(any("exactly one" in error for error in errors), errors)
        self.assertTrue(any("Commands run" in error for error in errors), errors)
        self.assertTrue(any("Evidence" in error for error in errors), errors)

    def test_rejects_missing_pull_request_body(self):
        """Reject pull_request events that do not provide a body string."""
        handle = tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False)
        json.dump({"pull_request": {"body": None}}, handle)
        handle.close()
        self.addCleanup(Path(handle.name).unlink, missing_ok=True)
        errors = validate_event(Path(handle.name))
        self.assertTrue(any("body" in error.lower() for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
