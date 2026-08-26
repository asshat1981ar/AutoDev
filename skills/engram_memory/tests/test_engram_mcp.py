import unittest
import sys
import os
import tempfile
import shutil

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from engram_mcp import EngramMemory


class TestEngramMemory(unittest.TestCase):
    """RED Phase: Tests for Engram MCP Server skill."""
    
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test-memory.db")
    
    def tearDown(self):
        if self.temp_dir:
            shutil.rmtree(self.temp_dir)
    
    def test_import(self):
        """Should import successfully."""
        self.assertIsNotNone(EngramMemory)
    
    def test_instantiation(self):
        """Should instantiate with custom DB path."""
        engram = EngramMemory(db_path=self.db_path)
        self.assertIsNotNone(engram)
    
    def test_store_and_retrieve(self):
        """Should store and retrieve memories with AMX compliance."""
        engram = EngramMemory(db_path=self.db_path)
        
        memory = {
            "logical_identity": "test-session-001",
            "origin": "test",
            "memory_type": "episode",
            "content": "Test memory content",
            "purpose": {"primary": "test", "task_class": "memory:amx"},
            "trust_validity_state": {"trust_level": "high", "validity_status": "valid"},
            "visibility": {"scope": "workspace"}
        }
        
        logical_id = engram.store(memory)
        self.assertEqual(logical_id, "test-session-001")
        
        retrieved = engram.retrieve("test-session-001")
        self.assertIsNotNone(retrieved)
        self.assertEqual(retrieved["logical_identity"], "test-session-001")
        self.assertIn("canonical_semantic_digest", retrieved)
    
    def test_query(self):
        """Should query memories."""
        engram = EngramMemory(db_path=self.db_path)
        
        for i in range(5):
            memory = {
                "logical_identity": f"test-{i}",
                "origin": "test",
                "memory_type": "episode",
                "content": f"Test memory {i}"
            }
            engram.store(memory)
        
        results = engram.query(memory_type="episode", limit=3)
        self.assertEqual(len(results), 3)
    
    def test_compute_digest(self):
        """Should compute SHA-256 digest."""
        engram = EngramMemory(db_path=self.db_path)
        data = {"key": "value", "number": 42}
        digest = engram.compute_digest(data)
        self.assertEqual(len(digest), 64)


if __name__ == '__main__':
    unittest.main()
