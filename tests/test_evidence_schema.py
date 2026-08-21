#!/usr/bin/env python3
"""
RED Phase Test: Evidence Schema Validation

This test expresses the desired behavior: Evidence records should validate
against a JSON schema that enforces all required fields from AMCX-1 spec.

Expected to FAIL initially (RED) because schema does not exist yet.

Run with: python3 -m unittest tests.test_evidence_schema -v
"""

import json
import unittest
from pathlib import Path


class TestEvidenceSchema(unittest.TestCase):
    """Test suite for Evidence schema validation."""

    @classmethod
    def setUpClass(cls):
        cls.schema_path = Path(__file__).parent.parent / "schemas" / "evidence-record-v1.json"

    def test_evidence_schema_exists(self):
        """Schema file should exist."""
        self.assertTrue(
            self.schema_path.exists(),
            f"Evidence schema not found: {self.schema_path}"
        )

    def test_evidence_schema_valid_json(self):
        """Schema file should be valid JSON."""
        content = self.schema_path.read_text()
        json.loads(content)

    def test_evidence_schema_has_required_fields(self):
        """Schema should define required fields for evidence records."""
        schema = json.loads(self.schema_path.read_text())
        self.assertIn("required", schema, "Schema missing required array")
        
        required = schema["required"]
        self.assertIn("evidence_id", required, "evidence_id should be required")
        self.assertIn("evidence_type", required, "evidence_type should be required")
        self.assertIn("timestamp", required, "timestamp should be required")
        self.assertIn("checksum", required, "checksum should be required")

    def test_evidence_schema_has_freshness_field(self):
        """Schema should have freshness tracking field."""
        schema = json.loads(self.schema_path.read_text())
        properties = schema.get("properties", {})
        self.assertIn("freshness_timestamp", properties, "Missing freshness_timestamp")


if __name__ == "__main__":
    unittest.main()
