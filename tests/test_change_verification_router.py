import unittest

from scripts.change_verification_router import hook_response, verification_hints


class ChangeVerificationRouterTests(unittest.TestCase):
    def test_pr33_kotlin_change_requires_ktlint_before_push(self):
        hints = verification_hints(
            [
                ".github/workflows/ci.yml",
                "kotlin/android-command-center/src/main/kotlin/dev/autodev/commandcenter/MainActivity.kt",
            ]
        )

        self.assertIn(
            "cd kotlin && ./gradlew :android-command-center:ktlintCheck --no-daemon",
            hints,
        )
        self.assertIn("python scripts/check_harness_drift.py", hints)

    def test_post_tool_hook_is_advisory_and_does_not_run_or_block(self):
        response = hook_response(
            {
                "hookName": "PostToolUse",
                "postToolUse": {
                    "tool": "write_to_file",
                    "parameters": {
                        "path": "kotlin/android-command-center/src/main/kotlin/dev/autodev/commandcenter/MainActivity.kt"
                    },
                    "result": "written",
                    "success": True,
                    "durationMs": 12,
                },
            }
        )

        self.assertFalse(response["cancel"])
        self.assertEqual(response["errorMessage"], "")
        self.assertIn("ktlintCheck", response["contextModification"])
        self.assertNotIn("ktlintFormat", response["contextModification"])

    def test_non_mutating_tool_produces_no_verification_context(self):
        response = hook_response(
            {
                "hookName": "PostToolUse",
                "postToolUse": {
                    "tool": "read_file",
                    "parameters": {"path": "README.md"},
                    "result": "contents",
                    "success": True,
                    "durationMs": 2,
                },
            }
        )

        self.assertEqual(
            response,
            {"cancel": False, "contextModification": "", "errorMessage": ""},
        )


if __name__ == "__main__":
    unittest.main()
