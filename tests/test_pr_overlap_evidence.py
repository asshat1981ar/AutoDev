import unittest

from scripts.pr_overlap_evidence import analyze_pr_overlaps, classify_path


class PullRequestOverlapEvidenceTests(unittest.TestCase):
    def test_classify_path_marks_authority_and_workflow_surfaces(self):
        self.assertEqual(classify_path("crates/forge-core/src/lib.rs"), "authority-core")
        self.assertEqual(classify_path(".github/workflows/ci.yml"), "ci-governance")
        self.assertEqual(classify_path("docs/evaluation.md"), "documentation")
        self.assertEqual(classify_path("scripts/tool.py"), "general")

    def test_analyzer_is_deterministic_and_reports_only_real_collisions(self):
        changed_paths = {
            25: [
                "crates/forge-core/src/lib.rs",
                "crates/forge-core/src/architecture_lease.rs",
            ],
            19: [
                "crates/forge-core/src/lib.rs",
                ".github/workflows/ci.yml",
            ],
            20: [
                "scripts/workload_replay.py",
                "docs/performance/workload-replay-baseline.md",
            ],
        }

        report = analyze_pr_overlaps(changed_paths)

        self.assertEqual(report["schema_version"], 1)
        self.assertEqual(report["pull_requests"], [19, 20, 25])
        self.assertEqual(report["collision_count"], 1)
        self.assertEqual(
            report["collisions"],
            [
                {
                    "path": "crates/forge-core/src/lib.rs",
                    "classification": "authority-core",
                    "pull_requests": [19, 25],
                    "severity": "high",
                }
            ],
        )
        self.assertEqual(report, analyze_pr_overlaps(dict(reversed(list(changed_paths.items())))))

    def test_three_way_overlap_is_one_collision_with_all_prs(self):
        report = analyze_pr_overlaps(
            {
                1: ["README.md", "crates/forge-core/src/action.rs"],
                2: ["crates/forge-core/src/action.rs"],
                3: ["crates/forge-core/src/action.rs"],
            }
        )

        self.assertEqual(report["collision_count"], 1)
        collision = report["collisions"][0]
        self.assertEqual(collision["pull_requests"], [1, 2, 3])
        self.assertEqual(collision["severity"], "high")

    def test_duplicate_paths_within_one_pr_do_not_create_false_collision(self):
        report = analyze_pr_overlaps({1: ["README.md", "README.md"], 2: ["other.md"]})
        self.assertEqual(report["collision_count"], 0)
        self.assertEqual(report["collisions"], [])


if __name__ == "__main__":
    unittest.main()
