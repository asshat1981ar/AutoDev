# Architecture decision framework

For structural choices, score alternatives against:

1. authority-boundary preservation;
2. recoverability/crash consistency;
3. evidence/auditability;
4. Android/Termux feasibility;
5. multiplatform portability;
6. ecosystem interoperability;
7. implementation/reversal cost;
8. performance/resource cost;
9. testability/simulation quality;
10. developer/user comprehensibility.

Security/correctness constraints are vetoes, not merely weighted preferences. Among acceptable options, prefer the smallest reversible slice that produces measurable evidence.
