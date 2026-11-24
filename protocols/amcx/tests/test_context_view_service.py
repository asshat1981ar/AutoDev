"""AMCX context admission regression tests using canonical source DAGs."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))
from core.amx_dag import AMXEvent, AMXEventDAG
from ecm.context_view import ContextViewService
from ecm.scope_binding import ScopeBindingAuthority


def record_dag(ch, title, content, admission="ADMITTED"):
    dag = AMXEventDAG(ch * 64)
    dag.append_event(AMXEvent(
        "1" * 64, "RECORD_CREATED", dag.record_id, [], "producer",
        {"title": title, "content": content, "admission": admission},
        timestamp="2026-11-11T00:00:00+00:00",
    ))
    return dag


class TestContextViewService(unittest.TestCase):
    def setUp(self):
        self.authority = ScopeBindingAuthority(
            b"amcx-test-receiver-secret-minimum-32-bytes"
        )

    def service(self, *dags):
        bindings = [
            self.authority.issue(
                record_id=dag.record_id, dag_digest=dag.compute_dag_digest(),
                tenant_id="tenant-alpha", project_id="autodev-core",
                allowed_purposes=("DEV",),
            ) for dag in dags
        ]
        return ContextViewService(
            "tenant-alpha", "autodev-core", scope_authority=self.authority,
            scope_bindings=bindings,
        )

    def request(self, purpose="DEV"):
        return self.authority.issue_request(
            principal_id="trusted-user", tenant_id="tenant-alpha",
            project_id="autodev-core", task_id="task-1", purpose=purpose,
        )

    def heads(self, *dags):
        """Head attestations for the given DAGs, issued at call time.

        Computed from each DAG's current head, so a DAG mutated after
        its binding was issued is attested at its (mutated) head — the
        binding check is what rejects it, not the head check.
        """
        return [self.authority.issue_head_attestation(
            record_id=dag.record_id, head_digest=dag.compute_head_digest(),
        ) for dag in dags]

    def test_unauthorized_access_fails_closed(self):
        dag = record_dag("a", "Secret doc", "sensitive")
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=None,
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_quarantined_record_filtered_out(self):
        good = record_dag("a", "Valid record", "Safe content")
        bad = record_dag("b", "Quarantined record", "Suspicious", "QUARANTINED")
        view = self.service(good, bad).build_context_view(
            "task-1", [good, bad], request=self.request(),
            head_attestations=self.heads(good, bad),
        )
        self.assertEqual(len(view["admitted_records"]), 1)
        self.assertEqual(view["admitted_records"][0]["title"], "Valid record")
        self.assertEqual(view["rejected_count"], 1)

    def test_secret_containing_record_quarantined(self):
        dag = record_dag(
            "c", "Leaky Record", "Secret: sk-abcdef1234567890abcdef1234567890",
        )
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
            head_attestations=self.heads(dag),
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)



    def test_rejected_records_do_not_exhaust_positive_limit(self):
        bad = record_dag("a", "Rejected", "unsafe", "QUARANTINED")
        good = record_dag("b", "Allowed", "safe")
        extra = record_dag("c", "Extra", "safe")
        view = self.service(bad, good, extra).build_context_view(
            "task-1", [bad, good, extra], request=self.request(), max_records=1,
            head_attestations=self.heads(bad, good, extra),
        )
        self.assertEqual([r["record_id"] for r in view["admitted_records"]],
                         [good.record_id])
        self.assertEqual(view["rejected_count"], 1)

    def test_invalid_sources_do_not_prevent_later_valid_admission(self):
        empty = AMXEventDAG("a" * 64)
        good = record_dag("b", "Allowed", "safe")
        view = self.service(empty, good).build_context_view(
            "task-1", [None, {}, empty, good], request=self.request(),
            head_attestations=self.heads(good),
        )
        self.assertEqual([r["record_id"] for r in view["admitted_records"]],
                         [good.record_id])
        self.assertEqual(view["rejected_count"], 3)

    def test_all_exported_payload_fields_are_sanitized(self):
        credential = "sk-" + "x" * 24
        for field in ("title", "metadata", "author"):
            with self.subTest(field=field):
                dag = record_dag("a", "Allowed", "safe")
                event = dag.get_event("1" * 64)
                if field == "author":
                    event.actor = credential
                else:
                    event.payload_delta[field] = credential
                # Sign the actual source: rejection must be due to sanitization.
                event.event_digest = event.compute_digest()
                view = self.service(dag).build_context_view(
                    "task-1", [dag], request=self.request(),
                    head_attestations=self.heads(dag),
                )
                self.assertEqual(view["admitted_records"], [])
                self.assertEqual(view["rejected_count"], 1)

    def test_readability_gates_apply_even_to_bound_sources(self):
        for field, state in (("validity", "EXPIRED"), ("validity", "REVOKED"),
                             ("admission", "REJECTED"),
                             ("content_lifecycle", "RETRACTED")):
            with self.subTest(field=field, state=state):
                dag = record_dag("a", "Allowed", "safe")
                event = dag.get_event("1" * 64)
                event.payload_delta[field] = state
                event.event_digest = event.compute_digest()
                view = self.service(dag).build_context_view(
                    "task-1", [dag], request=self.request(),
                    head_attestations=self.heads(dag),
                )
                self.assertEqual(view["admitted_records"], [])
                self.assertEqual(view["rejected_count"], 1)

    # -- Stale-DAG head attestation (Option B) --

    def test_head_digest_changes_when_retraction_appended(self):
        dag = record_dag("a", "Allowed", "safe")
        before = dag.compute_head_digest()
        dag.append_event(AMXEvent(
            "2" * 64, "RECORD_RETRACTED", dag.record_id, ["1" * 64],
            "producer", {}, timestamp="2026-11-11T00:00:01+00:00",
        ))
        self.assertNotEqual(dag.compute_head_digest(), before)

    def test_pre_retraction_snapshot_rejected_when_head_has_moved(self):
        """The stale-DAG regression: binding matches, head does not.

        A snapshot taken before RECORD_RETRACTED has the same
        source_digest as the binding issued for it, so the binding
        check passes and the snapshot reads as CURRENT. The authority's
        head attestation names the live (post-retraction) head, which
        differs from the snapshot's head, so admission must reject it.
        """
        import copy
        live = record_dag("a", "Allowed", "safe")
        snapshot = copy.deepcopy(live)
        # The binding was issued for the snapshot's digest: it still matches.
        service = self.service(snapshot)
        live.append_event(AMXEvent(
            "2" * 64, "RECORD_RETRACTED", live.record_id, ["1" * 64],
            "producer", {}, timestamp="2026-11-11T00:00:01+00:00",
        ))
        # The authority attests the LIVE head, at admission time.
        attestation = self.authority.issue_head_attestation(
            record_id=live.record_id, head_digest=live.compute_head_digest(),
        )
        view = service.build_context_view(
            "task-1", [snapshot], request=self.request(),
            head_attestations=[attestation],
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_current_snapshot_admitted_with_matching_head_attestation(self):
        dag = record_dag("a", "Allowed", "safe")
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
            head_attestations=self.heads(dag),
        )
        self.assertEqual(len(view["admitted_records"]), 1)
        self.assertEqual(view["rejected_count"], 0)

    def test_missing_head_attestation_fails_closed(self):
        dag = record_dag("a", "Allowed", "safe")
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_head_attestation_for_other_record_rejected(self):
        dag = record_dag("a", "Allowed", "safe")
        other = record_dag("b", "Other", "safe")
        attestation = self.authority.issue_head_attestation(
            record_id=other.record_id, head_digest=other.compute_head_digest(),
        )
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
            head_attestations=[attestation],
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_expired_head_attestation_rejected(self):
        dag = record_dag("a", "Allowed", "safe")
        attestation = self.authority.issue_head_attestation(
            record_id=dag.record_id, head_digest=dag.compute_head_digest(),
            expires_at=1,
        )
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
            head_attestations=[attestation],
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

    def test_head_attestation_from_other_issuer_rejected(self):
        dag = record_dag("a", "Allowed", "safe")
        other_authority = ScopeBindingAuthority(
            b"another-independent-test-authority-key-32-bytes"
        )
        attestation = other_authority.issue_head_attestation(
            record_id=dag.record_id, head_digest=dag.compute_head_digest(),
        )
        view = self.service(dag).build_context_view(
            "task-1", [dag], request=self.request(),
            head_attestations=[attestation],
        )
        self.assertEqual(view["admitted_records"], [])
        self.assertEqual(view["rejected_count"], 1)

if __name__ == "__main__":
    unittest.main()
