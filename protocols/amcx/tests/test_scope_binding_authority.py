"""Unit contracts for receiver-owned signed bindings and read requests."""
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from ecm.scope_binding import ScopeBindingAuthority


class ScopeBindingAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.authority = ScopeBindingAuthority(b"unit-test-authority-key-32-bytes-minimum")
        self.binding_fields = dict(
            record_id="a" * 64, dag_digest="b" * 64,
            tenant_id="tenant", project_id="project", allowed_purposes=("DEV",),
        )
        self.request_fields = dict(
            principal_id="alice", tenant_id="tenant", project_id="project",
            task_id="task", purpose="DEV", expires_at=1001,
        )

    def test_secret_requires_at_least_32_bytes(self):
        for secret in (None, "x" * 32, b"x" * 31, bytearray(b"x" * 32)):
            with self.subTest(secret_type=type(secret)), self.assertRaises(ValueError):
                ScopeBindingAuthority(secret)
        self.assertTrue(ScopeBindingAuthority(b"x" * 32).verify(
            ScopeBindingAuthority(b"x" * 32).issue(**self.binding_fields)))

    def test_purposes_are_canonicalized_before_signing(self):
        first = self.authority.issue(**dict(
            self.binding_fields, allowed_purposes=["DEV", "AUDIT", "DEV"]))
        second = self.authority.issue(**dict(
            self.binding_fields, allowed_purposes=("AUDIT", "DEV")))
        self.assertEqual(first, second)
        self.assertEqual(first.allowed_purposes, ("AUDIT", "DEV"))
        self.assertTrue(self.authority.verify(first))

    def test_each_binding_field_is_authenticated(self):
        binding = self.authority.issue(**self.binding_fields)
        for field, value in dict(
            record_id="c" * 64, dag_digest="d" * 64, tenant_id="other",
            project_id="other", allowed_purposes=("AUDIT",), signature="0" * 64,
        ).items():
            with self.subTest(field=field):
                self.assertFalse(self.authority.verify(replace(binding, **{field: value})))

    def test_malformed_binding_fields_fail_closed(self):
        binding = self.authority.issue(**self.binding_fields)
        for field, values in {
            "record_id": ("A" * 64, "g" * 64, "a" * 63, None),
            "dag_digest": ("B" * 64, "", 123),
            "tenant_id": (" ", None), "project_id": ("", 123),
            "allowed_purposes": ((), ["DEV"], ("DEV", "DEV"), ("DEV", "AUDIT"), (" ",)),
            "signature": (None, b"bytes", "not-a-signature"),
        }.items():
            for value in values:
                with self.subTest(field=field, value=value):
                    self.assertFalse(self.authority.verify(replace(binding, **{field: value})))

    def test_request_expiry_is_exclusive_at_unix_second_boundary(self):
        request = self.authority.issue_request(**self.request_fields)
        for now, expected in ((1000, True), (1001, False), (1002, False)):
            with self.subTest(now=now), patch("ecm.scope_binding.time.time", return_value=now):
                self.assertEqual(self.authority.verify_request(request), expected)

    def test_default_request_lifetime_and_unique_nonces(self):
        fields = dict(self.request_fields)
        del fields["expires_at"]
        with patch("ecm.scope_binding.time.time", return_value=1000):
            first = self.authority.issue_request(**fields)
            second = self.authority.issue_request(**fields)
            self.assertEqual(first.expires_at, 1300)
            self.assertNotEqual(first.nonce, second.nonce)
            self.assertTrue(self.authority.verify_request(first))
            self.assertTrue(self.authority.verify_request(second))

    def test_every_request_field_is_authenticated(self):
        request = self.authority.issue_request(**self.request_fields)
        changes = dict(
            principal_id="bob", tenant_id="other", project_id="other",
            task_id="other", purpose="AUDIT", expires_at=2000,
            nonce="0" * 32 if request.nonce != "0" * 32 else "1" * 32,
            signature="0" * 64,
        )
        with patch("ecm.scope_binding.time.time", return_value=1000):
            for field, value in changes.items():
                with self.subTest(field=field):
                    self.assertFalse(self.authority.verify_request(replace(request, **{field: value})))
            other = ScopeBindingAuthority(b"different-unit-test-authority-key-long-enough")
            self.assertFalse(other.verify_request(request))

    def test_malformed_request_fields_fail_closed(self):
        request = self.authority.issue_request(**self.request_fields)
        for field, values in {
            "principal_id": (None, " "), "tenant_id": ("", 1),
            "project_id": (None, " "), "task_id": ("", False),
            "purpose": (" ", []), "expires_at": (True, 1001.0, "1001", 0, -1),
            "nonce": (None, "a" * 31, "A" * 32, "g" * 32),
            "signature": (None, b"bytes", "invalid"),
        }.items():
            for value in values:
                with self.subTest(field=field, value=value):
                    self.assertFalse(self.authority.verify_request(replace(request, **{field: value})))

    def test_untyped_objects_cannot_supply_proof(self):
        for proof in (None, True, {}, object()):
            with self.subTest(proof=proof):
                self.assertFalse(self.authority.verify(proof))
                self.assertFalse(self.authority.verify_request(proof))


if __name__ == "__main__":
    unittest.main()
