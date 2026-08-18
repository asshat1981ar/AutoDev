import json
import tempfile
import unittest
from pathlib import Path

from scripts.validate_toolset_memory import validate_file, validate_record


VALID_RECORD = {
    "schema_version": "toolset-pattern-v1",
    "pattern_id": "example-pattern-001",
    "task_class": "example task",
    "context": "test context",
    "combination": ["repository inspection", "TDD", "CI"],
    "result": "high",
    "evidence": ["focused test passed"],
    "strengths": ["deterministic"],
    "failure_modes": ["none observed"],
    "when_to_use": ["example tasks"],
    "when_not_to_use": ["unrelated tasks"],
    "confidence": "high",
    "sample_size": 1,
    "last_validated": "2026-08-18",
}


class ToolsetMemoryValidationTests(unittest.TestCase):
    def test_valid_record_has_no_errors(self):
        self.assertEqual(validate_record(dict(VALID_RECORD)), [])

    def test_invalid_slug_is_rejected(self):
        record = dict(VALID_RECORD, pattern_id="Bad Pattern")
        self.assertTrue(any("pattern_id" in error for error in validate_record(record)))

    def test_empty_combination_is_rejected(self):
        record = dict(VALID_RECORD, combination=[])
        self.assertTrue(any("combination" in error for error in validate_record(record)))

    def test_non_positive_sample_size_is_rejected(self):
        record = dict(VALID_RECORD, sample_size=0)
        self.assertTrue(any("sample_size" in error for error in validate_record(record)))

    def test_invalid_enums_and_date_are_rejected(self):
        record = dict(
            VALID_RECORD,
            result="excellent",
            confidence="certain",
            last_validated="18-08-2026",
        )
        errors = validate_record(record)
        self.assertTrue(any("result" in error for error in errors))
        self.assertTrue(any("confidence" in error for error in errors))
        self.assertTrue(any("last_validated" in error for error in errors))

    def test_file_rejects_duplicate_ids(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "patterns.jsonl"
            line = json.dumps(VALID_RECORD, sort_keys=True)
            path.write_text(f"{line}\n{line}\n", encoding="utf-8")
            errors = validate_file(path)
        self.assertTrue(any("duplicate pattern_id" in error for error in errors))

    def test_file_rejects_malformed_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "patterns.jsonl"
            path.write_text("{not-json}\n", encoding="utf-8")
            errors = validate_file(path)
        self.assertTrue(any("invalid JSON" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
