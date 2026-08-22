#!/usr/bin/env python3
"""
RED Phase Test: AMX Memory Schema Validation

This test expresses the desired behavior: AMX memory records should validate
against a JSON schema that enforces all required fields from AMCX-1 spec.

Expected to FAIL initially (RED) because schema does not exist yet.

Run with: python3 -m unittest tests.test_amx_memory_schema -v
"""

import json
import unittest
from pathlib import Path


# AMCX-1 v1.1 AMX Required Fields
AMX_REQUIRED_FIELDS = {
    "origin",
    "logical_identity", 
    "repository_scope",
    "provenance",
    "causal_ancestry",
    "trust_validity_state",
    "visibility",
    "purpose",
    "retraction_deletion_barriers",
    "canonical_semantic_digest",
}


class TestAMXMemorySchema(unittest.TestCase):
    """Test suite for AMX memory schema validation."""

    @classmethod
    def setUpClass(cls):
        cls.schema_path = Path(__file__).parent.parent / "schemas" / "amx-memory-v1.json"

    def test_amx_schema_exists(self):
        """Schema file should exist."""
        self.assertTrue(
            self.schema_path.exists(),
            f"AMX memory schema not found: {self.schema_path}"
        )

    def test_amx_schema_valid_json(self):
        """Schema file should be valid JSON."""
        content = self.schema_path.read_text()
        # This will raise JSONDecodeError if invalid
        json.loads(content)

    def test_amx_schema_has_required_properties(self):
        """Schema should define all AMX required properties."""
        schema = json.loads(self.schema_path.read_text())
        
        # Schema should have a properties section
        self.assertIn("properties", schema, "Schema missing properties")
        
        # All AMX required fields should be in properties
        properties = schema["properties"]
        for field in AMX_REQUIRED_FIELDS:
            self.assertIn(
                field, properties,
                f"Required AMX field '{field}' not in schema properties"
            )

    def test_amx_schema_required_array(self):
        """Schema should list all AMX required fields in required array."""
        schema = json.loads(self.schema_path.read_text())
        
        self.assertIn("required", schema, "Schema missing required array")
        
        required_fields = set(schema["required"])
        for field in AMX_REQUIRED_FIELDS:
            self.assertIn(
                field, required_fields,
                f"Required AMX field '{field}' not in schema required array"
            )

    def test_amx_schema_has_type_definitions(self):
        """Schema should define types for all properties."""
        schema = json.loads(self.schema_path.read_text())
        properties = schema["properties"]
        
        for field, field_schema in properties.items():
            self.assertIn(
                "type", field_schema,
                f"Property '{field}' missing type definition"
            )


if __name__ == "__main__":
    unittest.main()
