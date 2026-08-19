# Federated Harness Kernel architecture invariants

1. Agents propose intent; trusted policy authorizes effects.
2. Plans coordinate; they do not authorize.
3. External harness permissions are capability requests, not grants.
4. Verification is independent of the worker that claims success.
5. Required evidence fails closed when missing or unknown.
6. Retries/replans are finite and observable.
7. Interrupted uncertain effects reconcile before retry.
8. Android and companion clients are control surfaces, not privileged kernels.
9. Adaptive learning cannot self-expand authority.
10. Every promoted harness/plugin/profile configuration has reproducible provenance and evaluation evidence.
