import unittest
import json
import os
import sys

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / 'src'

class TestSchemaPayloadValidation(unittest.TestCase):
    def setUp(self):
        self.v1_dir = str(REPO_ROOT / 'registry' / 'v1')

    def test_json_schema_valid_syntax(self):
        for fname in os.listdir(self.v1_dir):
            if fname.endswith('.json'):
                path = os.path.join(self.v1_dir, fname)
                with open(path) as f:
                    schema_json = json.load(f)
                    self.assertIn('$schema', schema_json)
                    self.assertIn('title', schema_json)
                    self.assertIn('type', schema_json)

    def test_amx_record_properties(self):
        with open(os.path.join(self.v1_dir, 'amx.record.v1.json')) as f:
            s = json.load(f)
            props = s['properties']
            self.assertIn('record_id', props)
            self.assertIn('content_lifecycle', props)
            self.assertIn('admission', props)
            self.assertIn('validity', props)
            self.assertIn('provenance_digest', props)
            self.assertEqual(props['content_lifecycle']['enum'], ['PROPOSED', 'CURRENT', 'SUPERSEDED', 'RETRACTED'])
            self.assertEqual(props['admission']['enum'], ['UNASSESSED', 'QUARANTINED', 'ADMITTED', 'REJECTED'])

    def test_ecm_task_states(self):
        with open(os.path.join(self.v1_dir, 'ecm.task.v1.json')) as f:
            s = json.load(f)
            states = s['properties']['task_state']['enum']
            self.assertEqual(len(states), 16)
            self.assertIn('PROPOSED', states)
            self.assertIn('RUNNING', states)
            self.assertIn('COMPLETED', states)
            self.assertIn('MANUAL_REQUIRED', states)

    def test_gate_profile_properties(self):
        with open(os.path.join(self.v1_dir, 'amcx.gate_profile.v1.json')) as f:
            s = json.load(f)
            props = s['properties']
            self.assertIn('profile_id', props)
            self.assertIn('subject_kind', props)
            self.assertIn('promotion_target', props)
            self.assertEqual(props['promotion_target']['enum'], ['CANARY', 'PRODUCTION', 'DEPRECATED'])

if __name__ == '__main__':
    unittest.main()
