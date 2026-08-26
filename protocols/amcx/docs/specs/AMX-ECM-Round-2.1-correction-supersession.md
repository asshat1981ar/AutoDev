# AMX-1 / ECM Round 2.1 correction and supersession

Artifact version: `2.1.0-correction`  
Date: 2026-08-20  
Status: corrected synthesis candidate; no implementation authorized  
Change policy: AMX-1 and ECM sources are immutable for this round

## 1. Supersession metadata

| Field | Value |
|---|---|
| `supersedes` | `AMX-ECM-Round-2-DifferenceRecord-matrix.md` |
| Superseded artifact SHA-256 | `b3e36bc2fe7e85b6cc485806339e3124c7908734cd4c1c505e514f59c8527837` |
| `superseded_by` | `AMX-ECM-Round-2.1-correction-supersession.md` with detached SHA-256 manifest |
| Supersession scope | DifferenceRecord references, classifications, ownership, coverage, traceability, blockers, and synthesis-safety decision |
| Original preservation | The superseded report remains byte-for-byte unchanged |

Every `D-####` identifier is preserved. In §3 each corrected record declares `supersedes=Round2:D-####` and `superseded_by=Round2.1:D-####`. This artifact does not modify either source or retroactively rewrite the rejected artifact.

## 2. Immutable input verification

| Artifact | Expected and observed SHA-256 |
|---|---|
| AMX-1 source | `4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9` |
| AMX normalization | `c81aedb9528df2162e5c327f6479a89848e70bf85a3835b3d76b67e5b06dae52` |
| ECM source | `e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980` |
| ECM normalization | `9ddf7754d017384f4d26ef801eac333a8e2a4148ef3d276fd178a032c49c7810` |
| Original Round 2 matrix | `b3e36bc2fe7e85b6cc485806339e3124c7908734cd4c1c505e514f59c8527837` |

## 3. Corrected expanded DifferenceRecord registry

Coverage is derived mechanically from the explicit, expanded `amx_ids` and `ecm_ids` in this registry. Ranges are not used in the registry. Requirements may occur in multiple records. Each published requirement has exactly one declared primary record in §10, and that primary record must explicitly reference it here.

| DifferenceRecord | Explicit AMX references | Explicit ECM references | Classification | Semantic address | Supersession |
|---|---|---|---|---|---|
| D-0001 | AMX-R-0001, AMX-R-0002, AMX-R-0003, AMX-R-0004, AMX-R-0005, AMX-R-0006, AMX-R-0007, AMX-R-0008 | ECM-R-0001, ECM-R-0002, ECM-R-0003, ECM-R-0004, ECM-R-0005, ECM-R-0006, ECM-R-0007, ECM-R-0008, ECM-R-0009, ECM-R-0010 | complementary | AMX memory mission and ECM collaboration mission form separate, jointly required scopes. | supersedes=Round2:D-0001; superseded_by=Round2.1:D-0001 |
| D-0002 | AMX-R-0009, AMX-R-0010, AMX-R-0011, AMX-R-0012 | ECM-R-0011, ECM-R-0012 | complementary | Authorization-boundary, transcript, hidden-reasoning, sensitive-data and model-authority exclusions are jointly enforced. | supersedes=Round2:D-0002; superseded_by=Round2.1:D-0002 |
| D-0003 | AMX-R-0006, AMX-R-0012, AMX-R-0019, AMX-R-0023, AMX-R-0117, AMX-R-0118, AMX-R-0119, AMX-R-0120, AMX-R-0121, AMX-R-0122, AMX-R-0123 | ECM-R-0008, ECM-R-0013, ECM-R-0015, ECM-R-0020, ECM-R-0021, ECM-R-0022, ECM-R-0023, ECM-R-0024, ECM-R-0025, ECM-R-0107, ECM-R-0112, ECM-R-0162, ECM-R-0172, ECM-R-0178, ECM-R-0179, ECM-R-0203, ECM-R-0209, ECM-R-0217, ECM-R-0225, ECM-R-0237 | identical | Only external trusted policy/execution may grant authority; memory, messages, models, votes and adapters cannot. | supersedes=Round2:D-0003; superseded_by=Round2.1:D-0003 |
| D-0004 | AMX-R-0008, AMX-R-0013 | ECM-R-0009, ECM-R-0016, ECM-R-0018 | complementary | Both remain locally operable without mandatory specialized stores and bound CRDT/delegation complexity. | supersedes=Round2:D-0004; superseded_by=Round2.1:D-0004 |
| D-0005 | AMX-R-0002, AMX-R-0003, AMX-R-0021, AMX-R-0034, AMX-R-0046 | ECM-R-0002, ECM-R-0026, ECM-R-0027, ECM-R-0062, ECM-R-0068, ECM-R-0095, ECM-R-0160, ECM-R-0204 | complementary | Hard scope/authority/lifecycle filters precede ranking; ECM adds governed context-view metadata. | supersedes=Round2:D-0005; superseded_by=Round2.1:D-0005 |
| D-0006 | AMX-R-0004, AMX-R-0024, AMX-R-0054, AMX-R-0104, AMX-R-0105, AMX-R-0106, AMX-R-0107 | ECM-R-0003, ECM-R-0028, ECM-R-0089, ECM-R-0110, ECM-R-0111, ECM-R-0112, ECM-R-0206, ECM-R-0219 | identical | Contradictions remain visible and neither order nor majority may silently resolve them. | supersedes=Round2:D-0006; superseded_by=Round2.1:D-0006 |
| D-0007 | AMX-R-0020, AMX-R-0037, AMX-R-0048, AMX-R-0049, AMX-R-0178 | ECM-R-0103, ECM-R-0104, ECM-R-0105, ECM-R-0142, ECM-R-0205, ECM-R-0221 | complementary | AMX validity references compose with EvidenceStore-owned exact-subject verdict and freshness semantics. | supersedes=Round2:D-0007; superseded_by=Round2.1:D-0007 |
| D-0008 | AMX-R-0017, AMX-R-0061, AMX-R-0062, AMX-R-0063, AMX-R-0064, AMX-R-0065, AMX-R-0066, AMX-R-0067, AMX-R-0068, AMX-R-0069, AMX-R-0070, AMX-R-0127, AMX-R-0128 | ECM-R-0015, ECM-R-0017, ECM-R-0031, ECM-R-0133, ECM-R-0175, ECM-R-0176, ECM-R-0177, ECM-R-0178, ECM-R-0179, ECM-R-0180, ECM-R-0181, ECM-R-0182, ECM-R-0183, ECM-R-0184 | complementary | Source maps partition external owners; ECM collaboration and AMX memory hold references/projections only. | supersedes=Round2:D-0008; superseded_by=Round2.1:D-0008 |
| D-0009 | AMX-R-0026, AMX-R-0044, AMX-R-0045, AMX-R-0046, AMX-R-0047, AMX-R-0048, AMX-R-0049, AMX-R-0050, AMX-R-0051, AMX-R-0052, AMX-R-0053, AMX-R-0054, AMX-R-0055, AMX-R-0056, AMX-R-0057, AMX-R-0058, AMX-R-0059, AMX-R-0060, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083 | ECM-R-0036, ECM-R-0041, ECM-R-0042, ECM-R-0043, ECM-R-0044, ECM-R-0046, ECM-R-0103, ECM-R-0104, ECM-R-0124, ECM-R-0138, ECM-R-0174 | complementary | AMX memory canonicalization and ECM workflow/effect/trace identity complement each other but target canonicalization remains incomplete. | supersedes=Round2:D-0009; superseded_by=Round2.1:D-0009 |
| D-0010 | AMX-R-0071, AMX-R-0072, AMX-R-0073, AMX-R-0074, AMX-R-0075, AMX-R-0076, AMX-R-0077, AMX-R-0078, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083, AMX-R-0084, AMX-R-0085, AMX-R-0086, AMX-R-0087, AMX-R-0088, AMX-R-0089, AMX-R-0090, AMX-R-0091, AMX-R-0092, AMX-R-0093, AMX-R-0094 | ECM-R-0071, ECM-R-0072, ECM-R-0073, ECM-R-0074, ECM-R-0075, ECM-R-0076, ECM-R-0077, ECM-R-0078, ECM-R-0079, ECM-R-0080, ECM-R-0081, ECM-R-0082, ECM-R-0083, ECM-R-0084, ECM-R-0085, ECM-R-0086, ECM-R-0087, ECM-R-0088, ECM-R-0089, ECM-R-0090, ECM-R-0091, ECM-R-0092, ECM-R-0093, ECM-R-0094, ECM-R-0095, ECM-R-0096, ECM-R-0097, ECM-R-0098, ECM-R-0099, ECM-R-0100, ECM-R-0101, ECM-R-0102, ECM-R-0185, ECM-R-0195, ECM-R-0196 | conflict | AMX and evidence-memory-v1 overlap as canonical memory representations; option analysis selects AMX plus a noncanonical binding only if falsification passes. | supersedes=Round2:D-0010; superseded_by=Round2.1:D-0010 |
| D-0011 | AMX-R-0068, AMX-R-0077, AMX-R-0078, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083, AMX-R-0084, AMX-R-0095, AMX-R-0104, AMX-R-0105, AMX-R-0106, AMX-R-0107, AMX-R-0139 | ECM-R-0039, ECM-R-0040, ECM-R-0133, ECM-R-0166, ECM-R-0177, ECM-R-0200, ECM-R-0201, ECM-R-0226 | complementary | AMX owns memory causality; ECM owns collaboration delivery and references AMX digests. | supersedes=Round2:D-0011; superseded_by=Round2.1:D-0011 |
| D-0012 | AMX-R-0017, AMX-R-0064 | ECM-R-0031, ECM-R-0032, ECM-R-0033, ECM-R-0034, ECM-R-0035, ECM-R-0052, ECM-R-0053, ECM-R-0054, ECM-R-0055, ECM-R-0056, ECM-R-0057, ECM-R-0058, ECM-R-0137, ECM-R-0138, ECM-R-0139, ECM-R-0140, ECM-R-0141, ECM-R-0142, ECM-R-0176, ECM-R-0177, ECM-R-0178, ECM-R-0179, ECM-R-0200, ECM-R-0202 | complementary | ExecPlan, ECM collaboration, and ForgeCore effects are separate domains; cardinality/reduction remains blocked. | supersedes=Round2:D-0012; superseded_by=Round2.1:D-0012 |
| D-0013 | AMX-R-0028, AMX-R-0077, AMX-R-0078, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083, AMX-R-0098, AMX-R-0099, AMX-R-0100, AMX-R-0101, AMX-R-0102, AMX-R-0103, AMX-R-0115 | ECM-R-0065, ECM-R-0066, ECM-R-0067, ECM-R-0068, ECM-R-0069, ECM-R-0070, ECM-R-0071, ECM-R-0072, ECM-R-0073, ECM-R-0074, ECM-R-0075, ECM-R-0076, ECM-R-0077, ECM-R-0078, ECM-R-0079, ECM-R-0080, ECM-R-0081, ECM-R-0082, ECM-R-0083, ECM-R-0084, ECM-R-0085, ECM-R-0086, ECM-R-0087, ECM-R-0088, ECM-R-0089, ECM-R-0090, ECM-R-0091, ECM-R-0092, ECM-R-0093, ECM-R-0094, ECM-R-0095, ECM-R-0096, ECM-R-0097, ECM-R-0098, ECM-R-0099, ECM-R-0100, ECM-R-0117, ECM-R-0118, ECM-R-0119, ECM-R-0120, ECM-R-0121, ECM-R-0122, ECM-R-0123, ECM-R-0124, ECM-R-0125, ECM-R-0126, ECM-R-0127, ECM-R-0128, ECM-R-0232 | complementary | AMX grammar, ECM context/configuration workflows and external memory-governance decisions remain independent. | supersedes=Round2:D-0013; superseded_by=Round2.1:D-0013 |
| D-0014 | AMX-R-0003, AMX-R-0016, AMX-R-0115, AMX-R-0239 | ECM-R-0030, ECM-R-0073, ECM-R-0108, ECM-R-0121 | missing | ECM lacks a mandatory current scoped user-authorization rule for cross-project memory promotion. | supersedes=Round2:D-0014; superseded_by=Round2.1:D-0014 |
| D-0015 | AMX-R-0022, AMX-R-0029, AMX-R-0032, AMX-R-0033, AMX-R-0036, AMX-R-0050, AMX-R-0051, AMX-R-0098, AMX-R-0099, AMX-R-0215, AMX-R-0216 | ECM-R-0022, ECM-R-0067, ECM-R-0069, ECM-R-0074, ECM-R-0082, ECM-R-0086, ECM-R-0087, ECM-R-0162 | missing | ECM lacks receiver binding and externally authorized quarantine-release/trust-widening guards. | supersedes=Round2:D-0015; superseded_by=Round2.1:D-0015 |
| D-0016 | AMX-R-0014, AMX-R-0059, AMX-R-0096, AMX-R-0097, AMX-R-0108, AMX-R-0109, AMX-R-0110, AMX-R-0183, AMX-R-0193 | ECM-R-0092, ECM-R-0095, ECM-R-0133, ECM-R-0153, ECM-R-0158, ECM-R-0208 | unsupported_or_unevidenced | Purge authorization, partial failure, receipts and anti-resurrection semantics are absent. | supersedes=Round2:D-0016; superseded_by=Round2.1:D-0016 |
| D-0017 | AMX-R-0073, AMX-R-0160, AMX-R-0193, AMX-R-0206 | ECM-R-0050, ECM-R-0159, ECM-R-0160, ECM-R-0161, ECM-R-0162, ECM-R-0163, ECM-R-0164, ECM-R-0165, ECM-R-0166, ECM-R-0167, ECM-R-0168, ECM-R-0169, ECM-R-0170, ECM-R-0171, ECM-R-0172, ECM-R-0173, ECM-R-0174, ECM-R-0209 | conflict | Unknown bytes must survive while semantic use of unknown critical extensions fails closed. | supersedes=Round2:D-0017; superseded_by=Round2.1:D-0017 |
| D-0018 | AMX-R-0021, AMX-R-0034, AMX-R-0046, AMX-R-0164, AMX-R-0175, AMX-R-0222, AMX-R-0223, AMX-R-0224, AMX-R-0225, AMX-R-0226, AMX-R-0227, AMX-R-0228, AMX-R-0229, AMX-R-0230, AMX-R-0231 | ECM-R-0027, ECM-R-0046, ECM-R-0051, ECM-R-0055, ECM-R-0062, ECM-R-0068, ECM-R-0095, ECM-R-0142, ECM-R-0160, ECM-R-0204 | complementary | ECM owns context replay; AMX supplies memory eligibility; current external leases/identity/evidence are revalidated. | supersedes=Round2:D-0018; superseded_by=Round2.1:D-0018 |
| D-0019 | AMX-R-0165, AMX-R-0166, AMX-R-0167, AMX-R-0168, AMX-R-0169, AMX-R-0170, AMX-R-0171, AMX-R-0172, AMX-R-0173, AMX-R-0174, AMX-R-0175, AMX-R-0176, AMX-R-0177, AMX-R-0178, AMX-R-0179, AMX-R-0180, AMX-R-0181, AMX-R-0182, AMX-R-0183, AMX-R-0184, AMX-R-0185 | ECM-R-0018, ECM-R-0019, ECM-R-0024, ECM-R-0030, ECM-R-0057, ECM-R-0058, ECM-R-0108, ECM-R-0118, ECM-R-0119, ECM-R-0120, ECM-R-0121, ECM-R-0122, ECM-R-0123, ECM-R-0124, ECM-R-0125, ECM-R-0126, ECM-R-0127, ECM-R-0128, ECM-R-0129, ECM-R-0130, ECM-R-0131, ECM-R-0132, ECM-R-0154, ECM-R-0156, ECM-R-0157, ECM-R-0210, ECM-R-0220, ECM-R-0223, ECM-R-0224, ECM-R-0236 | conflict | Subject-specific GateProfiles must reconcile efficacy/cost predicates, verifier relationships and hierarchical budgets. | supersedes=Round2:D-0019; superseded_by=Round2.1:D-0019 |
| D-0020 | AMX-R-0044, AMX-R-0045, AMX-R-0046, AMX-R-0047, AMX-R-0048, AMX-R-0049, AMX-R-0050, AMX-R-0051, AMX-R-0052, AMX-R-0053, AMX-R-0054, AMX-R-0055 | ECM-R-0071, ECM-R-0072, ECM-R-0073, ECM-R-0074, ECM-R-0075, ECM-R-0076, ECM-R-0077, ECM-R-0078, ECM-R-0079, ECM-R-0080, ECM-R-0081 | complementary | AMX axes and ECM function/visibility/trust taxonomy compose only as representation plus governed projections. | supersedes=Round2:D-0020; superseded_by=Round2.1:D-0020 |
| D-0021 | AMX-R-0027, AMX-R-0028, AMX-R-0098, AMX-R-0099, AMX-R-0100, AMX-R-0101, AMX-R-0102, AMX-R-0103, AMX-R-0210, AMX-R-0211, AMX-R-0212, AMX-R-0213, AMX-R-0214, AMX-R-0215, AMX-R-0216, AMX-R-0217, AMX-R-0218, AMX-R-0219, AMX-R-0220, AMX-R-0221 | ECM-R-0067, ECM-R-0069, ECM-R-0070, ECM-R-0086, ECM-R-0087, ECM-R-0088, ECM-R-0089, ECM-R-0090, ECM-R-0091, ECM-R-0092, ECM-R-0093, ECM-R-0229 | equivalent_rename | Write stages align, while authorization and secret prohibition remain externally enforced. | supersedes=Round2:D-0021; superseded_by=Round2.1:D-0021 |
| D-0022 | AMX-R-0025, AMX-R-0123, AMX-R-0164, AMX-R-0181, AMX-R-0222, AMX-R-0223, AMX-R-0224, AMX-R-0225, AMX-R-0226, AMX-R-0227, AMX-R-0228, AMX-R-0229, AMX-R-0230, AMX-R-0231 | ECM-R-0026, ECM-R-0068, ECM-R-0094, ECM-R-0095, ECM-R-0096, ECM-R-0097, ECM-R-0098, ECM-R-0099, ECM-R-0100, ECM-R-0144, ECM-R-0204, ECM-R-0231 | equivalent_rename | Retrieval stages align; ECM assembles context and AMX governs memory eligibility. | supersedes=Round2:D-0022; superseded_by=Round2.1:D-0022 |
| D-0023 | AMX-R-0141, AMX-R-0142, AMX-R-0143, AMX-R-0144, AMX-R-0145, AMX-R-0146, AMX-R-0147 | ECM-R-0037, ECM-R-0038, ECM-R-0163 | equivalent_rename | MCP remains a disposable capability adapter and never canonical state or authority. | supersedes=Round2:D-0023; superseded_by=Round2.1:D-0023 |
| D-0024 | AMX-R-0148, AMX-R-0149, AMX-R-0150, AMX-R-0151, AMX-R-0152, AMX-R-0153, AMX-R-0154 | ECM-R-0036, ECM-R-0040, ECM-R-0168 | complementary | AMX memory artifacts and ECM A2A collaboration/gateway/lease semantics compose. | supersedes=Round2:D-0024; superseded_by=Round2.1:D-0024 |
| D-0025 | AMX-R-0015, AMX-R-0061, AMX-R-0065, AMX-R-0124, AMX-R-0125, AMX-R-0126, AMX-R-0134, AMX-R-0135, AMX-R-0136, AMX-R-0137, AMX-R-0163, AMX-R-0243 | ECM-R-0167, ECM-R-0171, ECM-R-0172, ECM-R-0173, ECM-R-0175, ECM-R-0182 | complementary | Reviewed repository/instruction integration remains advisory and cannot auto-publish runtime memory. | supersedes=Round2:D-0025; superseded_by=Round2.1:D-0025 |
| D-0026 | AMX-R-0049, AMX-R-0089, AMX-R-0090 | ECM-R-0103, ECM-R-0104, ECM-R-0105, ECM-R-0106, ECM-R-0135, ECM-R-0180, ECM-R-0181, ECM-R-0221 | missing | AMX lacks complete artifact/evidence contracts; EvidenceStore/ArtifactStore remain canonical external owners. | supersedes=Round2:D-0026; superseded_by=Round2.1:D-0026 |
| D-0027 | — | ECM-R-0045, ECM-R-0046, ECM-R-0047, ECM-R-0048, ECM-R-0049, ECM-R-0050, ECM-R-0051, ECM-R-0052, ECM-R-0053, ECM-R-0054, ECM-R-0055, ECM-R-0056, ECM-R-0057, ECM-R-0058, ECM-R-0059, ECM-R-0060, ECM-R-0061, ECM-R-0062, ECM-R-0063, ECM-R-0064, ECM-R-0065, ECM-R-0066, ECM-R-0067, ECM-R-0068, ECM-R-0069, ECM-R-0070 | missing | AMX intentionally lacks ECM collaboration envelopes, task/role/cross-prompt and task-context contracts. | supersedes=Round2:D-0027; superseded_by=Round2.1:D-0027 |
| D-0028 | AMX-R-0024, AMX-R-0033 | ECM-R-0107, ECM-R-0108, ECM-R-0109, ECM-R-0110, ECM-R-0111, ECM-R-0112, ECM-R-0113, ECM-R-0114, ECM-R-0115, ECM-R-0116, ECM-R-0117, ECM-R-0118, ECM-R-0123, ECM-R-0236, ECM-R-0237, ECM-R-0238 | complementary | AMX conflict/corroboration controls complement ECM decision, retrospective and improvement governance. | supersedes=Round2:D-0028; superseded_by=Round2.1:D-0028 |
| D-0029 | AMX-R-0095, AMX-R-0104, AMX-R-0105, AMX-R-0106, AMX-R-0107, AMX-R-0108, AMX-R-0109, AMX-R-0127, AMX-R-0128, AMX-R-0129, AMX-R-0130, AMX-R-0131, AMX-R-0132, AMX-R-0133, AMX-R-0134, AMX-R-0135, AMX-R-0136, AMX-R-0137, AMX-R-0138, AMX-R-0139, AMX-R-0140 | ECM-R-0029, ECM-R-0133, ECM-R-0134, ECM-R-0135, ECM-R-0136, ECM-R-0137, ECM-R-0138, ECM-R-0139, ECM-R-0140, ECM-R-0141, ECM-R-0142, ECM-R-0177, ECM-R-0179, ECM-R-0200, ECM-R-0202, ECM-R-0227 | conflict | Memory and collaboration persistence are distinct, but ECM effect lifecycle conflicts with ForgeCore ownership until defined as projection. | supersedes=Round2:D-0029; superseded_by=Round2.1:D-0029 |
| D-0030 | AMX-R-0043, AMX-R-0055, AMX-R-0169, AMX-R-0170, AMX-R-0171, AMX-R-0172 | ECM-R-0041, ECM-R-0042, ECM-R-0043, ECM-R-0044, ECM-R-0143, ECM-R-0144, ECM-R-0145, ECM-R-0146, ECM-R-0147, ECM-R-0148, ECM-R-0226 | complementary | Memory influence/lifecycle telemetry and ECM run/tool/evidence telemetry compose under a non-authoritative audit plane. | supersedes=Round2:D-0030; superseded_by=Round2.1:D-0030 |
| D-0031 | AMX-R-0029, AMX-R-0030, AMX-R-0031, AMX-R-0032, AMX-R-0033, AMX-R-0034, AMX-R-0035, AMX-R-0036, AMX-R-0037, AMX-R-0038, AMX-R-0039, AMX-R-0040, AMX-R-0041, AMX-R-0042, AMX-R-0043, AMX-R-0110 | ECM-R-0149, ECM-R-0150, ECM-R-0151, ECM-R-0152, ECM-R-0153, ECM-R-0154, ECM-R-0155, ECM-R-0156, ECM-R-0157, ECM-R-0158, ECM-R-0241, ECM-R-0242 | complementary | Threat suites compose but severity-scoped recovery is missing. | supersedes=Round2:D-0031; superseded_by=Round2.1:D-0031 |
| D-0032 | AMX-R-0129, AMX-R-0130, AMX-R-0131, AMX-R-0132, AMX-R-0133, AMX-R-0155, AMX-R-0156, AMX-R-0157, AMX-R-0158, AMX-R-0159, AMX-R-0160, AMX-R-0161 | ECM-R-0020, ECM-R-0159, ECM-R-0160, ECM-R-0161, ECM-R-0162, ECM-R-0163, ECM-R-0164, ECM-R-0165, ECM-R-0166, ECM-R-0167, ECM-R-0168, ECM-R-0169, ECM-R-0170, ECM-R-0171, ECM-R-0172, ECM-R-0173, ECM-R-0174, ECM-R-0183 | complementary | Memory projections and harness adapters compose, but delivery/resume/cancel/downgrade semantics are incomplete. | supersedes=Round2:D-0032; superseded_by=Round2.1:D-0032 |
| D-0033 | AMX-R-0186, AMX-R-0187, AMX-R-0188, AMX-R-0189, AMX-R-0190, AMX-R-0191, AMX-R-0192, AMX-R-0193 | ECM-R-0101, ECM-R-0102, ECM-R-0153, ECM-R-0208, ECM-R-0209 | missing | ECM lacks complete migration/provider-exit semantics and must reference external deletion and registry authority. | supersedes=Round2:D-0033; superseded_by=Round2.1:D-0033 |
| D-0034 | AMX-R-0018, AMX-R-0111, AMX-R-0112, AMX-R-0113, AMX-R-0114, AMX-R-0115, AMX-R-0116, AMX-R-0185, AMX-R-0194, AMX-R-0195, AMX-R-0196, AMX-R-0197, AMX-R-0198, AMX-R-0199, AMX-R-0200, AMX-R-0201, AMX-R-0202, AMX-R-0203, AMX-R-0204, AMX-R-0205, AMX-R-0206, AMX-R-0207, AMX-R-0208, AMX-R-0209, AMX-R-0242, AMX-R-0243, AMX-R-0244 | ECM-R-0185, ECM-R-0186, ECM-R-0187, ECM-R-0188, ECM-R-0189, ECM-R-0190, ECM-R-0191, ECM-R-0192, ECM-R-0193, ECM-R-0194, ECM-R-0195, ECM-R-0196, ECM-R-0197, ECM-R-0198, ECM-R-0199, ECM-R-0200, ECM-R-0201, ECM-R-0202, ECM-R-0203, ECM-R-0204, ECM-R-0205, ECM-R-0206, ECM-R-0207, ECM-R-0208, ECM-R-0209, ECM-R-0210 | complementary | Contract-first sequencing aligns, subject to Registry activation and a machine-readable AcceptanceContract. | supersedes=Round2:D-0034; superseded_by=Round2.1:D-0034 |
| D-0035 | AMX-R-0158, AMX-R-0159, AMX-R-0163, AMX-R-0232, AMX-R-0233, AMX-R-0234, AMX-R-0235, AMX-R-0236, AMX-R-0237, AMX-R-0238, AMX-R-0239, AMX-R-0240, AMX-R-0241 | ECM-R-0014, ECM-R-0211, ECM-R-0212, ECM-R-0213, ECM-R-0214, ECM-R-0215, ECM-R-0216, ECM-R-0217, ECM-R-0218, ECM-R-0219, ECM-R-0220, ECM-R-0221, ECM-R-0222, ECM-R-0223, ECM-R-0224, ECM-R-0225, ECM-R-0226, ECM-R-0227, ECM-R-0228, ECM-R-0229, ECM-R-0230, ECM-R-0231, ECM-R-0232, ECM-R-0233, ECM-R-0234, ECM-R-0235, ECM-R-0236, ECM-R-0237, ECM-R-0238, ECM-R-0239, ECM-R-0240, ECM-R-0241, ECM-R-0242, ECM-R-0243, ECM-R-0244 | complementary | AMX onboarding and ECM controller behavior compose while controller duplicates remain explicitly related. | supersedes=Round2:D-0035; superseded_by=Round2.1:D-0035 |
| D-0036 | AMX-R-0001, AMX-R-0002, AMX-R-0003, AMX-R-0004, AMX-R-0005, AMX-R-0006, AMX-R-0007, AMX-R-0008, AMX-R-0009, AMX-R-0010, AMX-R-0011, AMX-R-0012, AMX-R-0013, AMX-R-0014, AMX-R-0015, AMX-R-0016, AMX-R-0017, AMX-R-0018, AMX-R-0019, AMX-R-0020, AMX-R-0021, AMX-R-0022, AMX-R-0023, AMX-R-0024, AMX-R-0025, AMX-R-0026, AMX-R-0027, AMX-R-0028, AMX-R-0029, AMX-R-0030, AMX-R-0031, AMX-R-0032, AMX-R-0033, AMX-R-0034, AMX-R-0035, AMX-R-0036, AMX-R-0037, AMX-R-0038, AMX-R-0039, AMX-R-0040, AMX-R-0041, AMX-R-0042, AMX-R-0043, AMX-R-0044, AMX-R-0045, AMX-R-0046, AMX-R-0047, AMX-R-0048, AMX-R-0049, AMX-R-0050, AMX-R-0051, AMX-R-0052, AMX-R-0053, AMX-R-0054, AMX-R-0055, AMX-R-0056, AMX-R-0057, AMX-R-0058, AMX-R-0059, AMX-R-0060, AMX-R-0061, AMX-R-0062, AMX-R-0063, AMX-R-0064, AMX-R-0065, AMX-R-0066, AMX-R-0067, AMX-R-0068, AMX-R-0069, AMX-R-0070, AMX-R-0071, AMX-R-0072, AMX-R-0073, AMX-R-0074, AMX-R-0075, AMX-R-0076, AMX-R-0077, AMX-R-0078, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083, AMX-R-0084, AMX-R-0085, AMX-R-0086, AMX-R-0087, AMX-R-0088, AMX-R-0089, AMX-R-0090, AMX-R-0091, AMX-R-0092, AMX-R-0093, AMX-R-0094, AMX-R-0095, AMX-R-0096, AMX-R-0097, AMX-R-0098, AMX-R-0099, AMX-R-0100, AMX-R-0101, AMX-R-0102, AMX-R-0103, AMX-R-0104, AMX-R-0105, AMX-R-0106, AMX-R-0107, AMX-R-0108, AMX-R-0109, AMX-R-0110, AMX-R-0111, AMX-R-0112, AMX-R-0113, AMX-R-0114, AMX-R-0115, AMX-R-0116, AMX-R-0117, AMX-R-0118, AMX-R-0119, AMX-R-0120, AMX-R-0121, AMX-R-0122, AMX-R-0123, AMX-R-0124, AMX-R-0125, AMX-R-0126, AMX-R-0127, AMX-R-0128, AMX-R-0129, AMX-R-0130, AMX-R-0131, AMX-R-0132, AMX-R-0133, AMX-R-0134, AMX-R-0135, AMX-R-0136, AMX-R-0137, AMX-R-0138, AMX-R-0139, AMX-R-0140, AMX-R-0141, AMX-R-0142, AMX-R-0143, AMX-R-0144, AMX-R-0145, AMX-R-0146, AMX-R-0147, AMX-R-0148, AMX-R-0149, AMX-R-0150, AMX-R-0151, AMX-R-0152, AMX-R-0153, AMX-R-0154, AMX-R-0155, AMX-R-0156, AMX-R-0157, AMX-R-0158, AMX-R-0159, AMX-R-0160, AMX-R-0161, AMX-R-0163, AMX-R-0164, AMX-R-0165, AMX-R-0166, AMX-R-0167, AMX-R-0168, AMX-R-0169, AMX-R-0170, AMX-R-0171, AMX-R-0172, AMX-R-0173, AMX-R-0174, AMX-R-0175, AMX-R-0176, AMX-R-0177, AMX-R-0178, AMX-R-0179, AMX-R-0180, AMX-R-0181, AMX-R-0182, AMX-R-0183, AMX-R-0184, AMX-R-0185, AMX-R-0186, AMX-R-0187, AMX-R-0188, AMX-R-0189, AMX-R-0190, AMX-R-0191, AMX-R-0192, AMX-R-0193, AMX-R-0194, AMX-R-0195, AMX-R-0196, AMX-R-0197, AMX-R-0198, AMX-R-0199, AMX-R-0200, AMX-R-0201, AMX-R-0202, AMX-R-0203, AMX-R-0204, AMX-R-0205, AMX-R-0206, AMX-R-0207, AMX-R-0208, AMX-R-0209, AMX-R-0210, AMX-R-0211, AMX-R-0212, AMX-R-0213, AMX-R-0214, AMX-R-0215, AMX-R-0216, AMX-R-0217, AMX-R-0218, AMX-R-0219, AMX-R-0220, AMX-R-0221, AMX-R-0222, AMX-R-0223, AMX-R-0224, AMX-R-0225, AMX-R-0226, AMX-R-0227, AMX-R-0228, AMX-R-0229, AMX-R-0230, AMX-R-0231, AMX-R-0232, AMX-R-0233, AMX-R-0234, AMX-R-0235, AMX-R-0236, AMX-R-0237, AMX-R-0238, AMX-R-0239, AMX-R-0240, AMX-R-0241, AMX-R-0242, AMX-R-0243, AMX-R-0244 | — | unsupported_or_unevidenced | AMX normalization requires per-row modality/extraction/traceability metadata; AMX-R-0162 remains absent. | supersedes=Round2:D-0036; superseded_by=Round2.1:D-0036 |
| D-0037 | — | ECM-R-0001, ECM-R-0002, ECM-R-0003, ECM-R-0004, ECM-R-0005, ECM-R-0006, ECM-R-0007, ECM-R-0008, ECM-R-0009, ECM-R-0010, ECM-R-0011, ECM-R-0012, ECM-R-0013, ECM-R-0014, ECM-R-0015, ECM-R-0016, ECM-R-0017, ECM-R-0018, ECM-R-0019, ECM-R-0020, ECM-R-0021, ECM-R-0022, ECM-R-0023, ECM-R-0024, ECM-R-0025, ECM-R-0026, ECM-R-0027, ECM-R-0028, ECM-R-0029, ECM-R-0030, ECM-R-0031, ECM-R-0032, ECM-R-0033, ECM-R-0034, ECM-R-0035, ECM-R-0036, ECM-R-0037, ECM-R-0038, ECM-R-0039, ECM-R-0040, ECM-R-0041, ECM-R-0042, ECM-R-0043, ECM-R-0044, ECM-R-0045, ECM-R-0046, ECM-R-0047, ECM-R-0048, ECM-R-0049, ECM-R-0050, ECM-R-0051, ECM-R-0052, ECM-R-0053, ECM-R-0054, ECM-R-0055, ECM-R-0056, ECM-R-0057, ECM-R-0058, ECM-R-0059, ECM-R-0060, ECM-R-0061, ECM-R-0062, ECM-R-0063, ECM-R-0064, ECM-R-0065, ECM-R-0066, ECM-R-0067, ECM-R-0068, ECM-R-0069, ECM-R-0070, ECM-R-0071, ECM-R-0072, ECM-R-0073, ECM-R-0074, ECM-R-0075, ECM-R-0076, ECM-R-0077, ECM-R-0078, ECM-R-0079, ECM-R-0080, ECM-R-0081, ECM-R-0082, ECM-R-0083, ECM-R-0084, ECM-R-0085, ECM-R-0086, ECM-R-0087, ECM-R-0088, ECM-R-0089, ECM-R-0090, ECM-R-0091, ECM-R-0092, ECM-R-0093, ECM-R-0094, ECM-R-0095, ECM-R-0096, ECM-R-0097, ECM-R-0098, ECM-R-0099, ECM-R-0100, ECM-R-0101, ECM-R-0102, ECM-R-0103, ECM-R-0104, ECM-R-0105, ECM-R-0106, ECM-R-0107, ECM-R-0108, ECM-R-0109, ECM-R-0110, ECM-R-0111, ECM-R-0112, ECM-R-0113, ECM-R-0114, ECM-R-0115, ECM-R-0116, ECM-R-0117, ECM-R-0118, ECM-R-0119, ECM-R-0120, ECM-R-0121, ECM-R-0122, ECM-R-0123, ECM-R-0124, ECM-R-0125, ECM-R-0126, ECM-R-0127, ECM-R-0128, ECM-R-0129, ECM-R-0130, ECM-R-0131, ECM-R-0132, ECM-R-0133, ECM-R-0134, ECM-R-0135, ECM-R-0136, ECM-R-0137, ECM-R-0138, ECM-R-0139, ECM-R-0140, ECM-R-0141, ECM-R-0142, ECM-R-0143, ECM-R-0144, ECM-R-0145, ECM-R-0146, ECM-R-0147, ECM-R-0148, ECM-R-0149, ECM-R-0150, ECM-R-0151, ECM-R-0152, ECM-R-0153, ECM-R-0154, ECM-R-0155, ECM-R-0156, ECM-R-0157, ECM-R-0158, ECM-R-0159, ECM-R-0160, ECM-R-0161, ECM-R-0162, ECM-R-0163, ECM-R-0164, ECM-R-0165, ECM-R-0166, ECM-R-0167, ECM-R-0168, ECM-R-0169, ECM-R-0170, ECM-R-0171, ECM-R-0172, ECM-R-0173, ECM-R-0174, ECM-R-0175, ECM-R-0176, ECM-R-0177, ECM-R-0178, ECM-R-0179, ECM-R-0180, ECM-R-0181, ECM-R-0182, ECM-R-0183, ECM-R-0184, ECM-R-0185, ECM-R-0186, ECM-R-0187, ECM-R-0188, ECM-R-0189, ECM-R-0190, ECM-R-0191, ECM-R-0192, ECM-R-0193, ECM-R-0194, ECM-R-0195, ECM-R-0196, ECM-R-0197, ECM-R-0198, ECM-R-0199, ECM-R-0200, ECM-R-0201, ECM-R-0202, ECM-R-0203, ECM-R-0204, ECM-R-0205, ECM-R-0206, ECM-R-0207, ECM-R-0208, ECM-R-0209, ECM-R-0210, ECM-R-0211, ECM-R-0212, ECM-R-0213, ECM-R-0214, ECM-R-0215, ECM-R-0216, ECM-R-0217, ECM-R-0218, ECM-R-0219, ECM-R-0220, ECM-R-0221, ECM-R-0222, ECM-R-0223, ECM-R-0224, ECM-R-0225, ECM-R-0226, ECM-R-0227, ECM-R-0228, ECM-R-0229, ECM-R-0230, ECM-R-0231, ECM-R-0232, ECM-R-0233, ECM-R-0234, ECM-R-0235, ECM-R-0236, ECM-R-0237, ECM-R-0238, ECM-R-0239, ECM-R-0240, ECM-R-0241, ECM-R-0242, ECM-R-0243, ECM-R-0244 | unsupported_or_unevidenced | ECM normalization requires per-row modality/extraction/traceability metadata. | supersedes=Round2:D-0037; superseded_by=Round2.1:D-0037 |
| D-0038 | AMX-R-0071, AMX-R-0072, AMX-R-0073, AMX-R-0074, AMX-R-0075, AMX-R-0076, AMX-R-0077, AMX-R-0078, AMX-R-0079, AMX-R-0080, AMX-R-0081, AMX-R-0082, AMX-R-0083, AMX-R-0084, AMX-R-0085, AMX-R-0086, AMX-R-0087, AMX-R-0088, AMX-R-0089, AMX-R-0090, AMX-R-0091, AMX-R-0092, AMX-R-0093, AMX-R-0094, AMX-R-0203, AMX-R-0204, AMX-R-0205, AMX-R-0206 | ECM-R-0045, ECM-R-0046, ECM-R-0047, ECM-R-0048, ECM-R-0049, ECM-R-0050, ECM-R-0051, ECM-R-0052, ECM-R-0053, ECM-R-0054, ECM-R-0055, ECM-R-0056, ECM-R-0057, ECM-R-0058, ECM-R-0059, ECM-R-0060, ECM-R-0061, ECM-R-0062, ECM-R-0063, ECM-R-0064, ECM-R-0065, ECM-R-0066, ECM-R-0067, ECM-R-0068, ECM-R-0069, ECM-R-0070, ECM-R-0071, ECM-R-0072, ECM-R-0073, ECM-R-0074, ECM-R-0075, ECM-R-0076, ECM-R-0077, ECM-R-0078, ECM-R-0079, ECM-R-0080, ECM-R-0081, ECM-R-0082, ECM-R-0083, ECM-R-0084, ECM-R-0085, ECM-R-0086, ECM-R-0087, ECM-R-0088, ECM-R-0089, ECM-R-0090, ECM-R-0091, ECM-R-0092, ECM-R-0093, ECM-R-0094, ECM-R-0095, ECM-R-0096, ECM-R-0097, ECM-R-0098, ECM-R-0099, ECM-R-0100, ECM-R-0101, ECM-R-0102, ECM-R-0103, ECM-R-0104, ECM-R-0105, ECM-R-0106, ECM-R-0185, ECM-R-0186, ECM-R-0187, ECM-R-0188, ECM-R-0189, ECM-R-0190, ECM-R-0191, ECM-R-0192, ECM-R-0193, ECM-R-0194, ECM-R-0195, ECM-R-0196, ECM-R-0197, ECM-R-0198 | unsupported_or_unevidenced | Conceptual contracts and partial lifecycles are not machine-readable or conformance-ready. | supersedes=Round2:D-0038; superseded_by=Round2.1:D-0038 |
| D-0039 | AMX-R-0100, AMX-R-0158, AMX-R-0164, AMX-R-0210, AMX-R-0211, AMX-R-0212, AMX-R-0213, AMX-R-0214, AMX-R-0215, AMX-R-0216, AMX-R-0217, AMX-R-0218, AMX-R-0219, AMX-R-0220, AMX-R-0221, AMX-R-0222, AMX-R-0223, AMX-R-0224, AMX-R-0225, AMX-R-0226, AMX-R-0227, AMX-R-0228, AMX-R-0229, AMX-R-0230, AMX-R-0231, AMX-R-0232, AMX-R-0233, AMX-R-0234, AMX-R-0235, AMX-R-0236, AMX-R-0237, AMX-R-0238, AMX-R-0239, AMX-R-0240, AMX-R-0241, AMX-R-0242, AMX-R-0243, AMX-R-0244 | ECM-R-0013, ECM-R-0020, ECM-R-0057, ECM-R-0123, ECM-R-0154, ECM-R-0155, ECM-R-0156, ECM-R-0157, ECM-R-0180, ECM-R-0181, ECM-R-0182, ECM-R-0183, ECM-R-0184, ECM-R-0185, ECM-R-0200, ECM-R-0201, ECM-R-0202, ECM-R-0203, ECM-R-0204, ECM-R-0205, ECM-R-0206, ECM-R-0207, ECM-R-0208, ECM-R-0209, ECM-R-0210, ECM-R-0211, ECM-R-0212, ECM-R-0213, ECM-R-0214, ECM-R-0215, ECM-R-0216, ECM-R-0217, ECM-R-0218, ECM-R-0219, ECM-R-0220, ECM-R-0221, ECM-R-0222, ECM-R-0223, ECM-R-0224, ECM-R-0225, ECM-R-0226, ECM-R-0227, ECM-R-0228, ECM-R-0229, ECM-R-0230, ECM-R-0231, ECM-R-0232, ECM-R-0233, ECM-R-0234, ECM-R-0235, ECM-R-0236, ECM-R-0237, ECM-R-0238, ECM-R-0239, ECM-R-0240, ECM-R-0241, ECM-R-0242, ECM-R-0243, ECM-R-0244 | unsupported_or_unevidenced | Aggregate, alias and refinement relationships require explicit catalog metadata. | supersedes=Round2:D-0039; superseded_by=Round2.1:D-0039 |
| D-0040 | AMX-R-0180, AMX-R-0182, AMX-R-0184 | ECM-R-0122, ECM-R-0129, ECM-R-0130, ECM-R-0131, ECM-R-0132, ECM-R-0154, ECM-R-0156, ECM-R-0157 | unsupported_or_unevidenced | Numeric thresholds, routing prediction, reviewer correlation and local limits remain uncalibrated. | supersedes=Round2:D-0040; superseded_by=Round2.1:D-0040 |


## 4. Re-adjudication of challenged records

| Record | Corrected adjudication and source evidence | Falsifiable test | Disposition |
|---|---|---|---|
| D-0001 | `complementary`. AMX goals require memory exchange, scope preservation, contradiction history, deterministic import, advisory authority, outcome evaluation and minimal infrastructure (AMX source L37–48). ECM goals require neutral collaboration, private/shared context, provenance, bounded cross-prompting, recovery, normalized evaluation, evidence-gated promotion, external authority, local operation and distributed compatibility (ECM L19–32). Every AMX-R-0001–0008 and ECM-R-0001–0010 is explicitly addressed; none is treated as an alias. | Construct one acceptance vector per listed goal. A composition passes only if all 18 vectors are satisfied without one protocol claiming the other’s state. | `merge` |
| D-0002 | Corrected from `identical` to `complementary`. AMX-R-0009–0012 cover authorization-boundary crawling, proprietary-chat synchronization, prohibited stored data, and model-output authority (AMX L50–57). ECM-R-0011–0012 cover hidden reasoning and raw transcripts (ECM L34–37). AMX-R-0013–0018 and ECM-R-0013–0020 are no longer falsely assigned here. | Supply an unauthorized repository, proprietary transcript, secret-bearing record, hidden reasoning, and confident signed model claim. Each must be rejected for its own stated reason. | `merge` |
| D-0005 | `identical` on filter precedence, `complementary` on context metadata; registry classification remains `complementary`. AMX L41–43, L69 and L447 require repository/project/user/task/path/time/sensitivity/retention boundaries before rank. ECM L56–57, L293 and L520 require authorization/tenant/project/validity/deletion filters and digest/expiry/budget context views. | Vary tenant, project, path, task, validity, sensitivity, deletion, authority, and context expiry one at a time. Ranking must never see a disallowed record. | `merge` |
| D-0006 | `identical`. AMX L44, L72, L319–324 and ECM L58, L283, L331–340 preserve contradictions and prohibit silent semantic overwrite or majority-as-truth. | Generate concurrent contradictory claims. Neither clock order, import order, model count, nor majority may erase either claim; deterministic policy is required for resolution. | `keep` |
| D-0007 | `complementary`. AMX owns memory validity/provenance fields and current-evidence precedence (AMX L68, L125–126, L198–215, L450). EvidenceStore/VerificationFabric—not AMX or ECM—owns verdict and freshness. ECM’s ArtifactRef/EvidenceRef and stale-subject rules are reference contracts (ECM L305–327, L454, L625–626). | Change artifact digest, repository revision, toolchain, environment, evaluator, prompt, skill, permission, and validity independently. The evidence owner must mark the verdict stale and every projection must observe it. | `merge` |
| D-0008 | `complementary`, with corrected ownership. AMX’s source map is descriptive about external owners (AMX L78–91); ECM’s source map is also descriptive (ECM L578–591). AMX does not own identity, evidence truth, approval, effects, schema activation, or purge authorization. ECM owns only collaboration state and references/projections for external domains. | Delete every derived projection and rebuild it only from its declared canonical owner. Any projection that cannot rebuild or that can mutate the owner fails. | `merge` |
| D-0009 | `complementary` but incomplete. AMX provides memory repository identity, UUIDv7, timestamp and JCS/SHA-256 rules (AMX L273–283). ECM provides workflow/trace/effect identifiers and attestation references (ECM L84–104, L452). Neither defines complete repository-worktree or resolved effect-target canonicalization with cross-language vectors. | Rust, Kotlin, Go and TypeScript canonicalizers must produce identical repository, worktree, path, target, argument, record, event and effect identities—or identical deterministic errors—for the same fixtures. | `defer` |
| D-0011 | `complementary` only under strict ownership. AMX owns legal memory transitions, event history, causal parents and heads (AMX L233–255, L317–328). ECM owns collaboration delivery/reduction (ECM L432–441, L584). ECM workflow events may reference AMX digests but may not synthesize memory events or become their canonical log. | Crash before and after AMX commit, ECM outbox commit, delivery and acknowledgement. Duplicate/reordered replay must yield one AMX transition and identical ECM projections without dual authoritative writes. | `merge` |
| D-0012 | `complementary` with two new blockers. ExecPlan owns plans/steps (AMX L85; ECM L583). ECM owns collaboration tasks/attempts/roles/messages (ECM L150–174, L584). ForgeCore owns effects/receipts (ECM L585–586). ECM’s effect lifecycle is a projection of ForgeCore, not a second owner. | Map zero, one and many ECM tasks to one ExecPlan step; then vary task completion, cancellation, retry and partial success. ExecPlan completion must follow a declared reducer, and no ECM event may commit an effect without a ForgeCore receipt. | `defer` |
| D-0013 | `complementary` with corrected authority. AMX owns the legal grammar and history of memory admission/verification states. External memory-governance policy authorizes quarantine release, trust changes and visibility widening. ECM context admission owns task-context entries; ECM promotion owns prompt/skill/router/schema candidate evaluation, but only the Neutral Contract Registry can activate schemas. | Verify a memory, widen visibility, release quarantine, activate a prompt, and activate a schema independently. No operation may imply another; every authority decision must come from its named owner. | `merge` |
| D-0018 | `complementary` but under-specified. AMX defines memory eligibility filters; ECM owns context-view construction, attempt/task binding and lease-aware replay. Identity, capability and approval are queried from current external authorities. | Replay one context digest after changing tenant, project, repository, branch, path, task, attempt, role lease, capability lease, approval, expiry, evidence freshness, quarantine or deletion. Each relevant change must reject or rebuild before use. | `defer` |
| D-0020 | `complementary`. AMX owns canonical representation and transition grammar. ECM’s function/visibility/trust classifications (ECM L227–243) are views or policy inputs, not authorities. Trust and visibility transitions require external memory-governance authorization; EvidenceStore supplies verdict references. | Import an AMX record through each ECM class/visibility/trust label. Altering a label must not alter canonical AMX bytes or widen trust/visibility without policy authorization. | `merge` |
| D-0021 | `equivalent_rename` for the mechanical write stages, but authority is not equivalent. AMX L285–300 and ECM L278–288 align on validation, dedupe, contradiction preservation, bounds, artifacts and audit. External policy authorizes write scope; raw secrets, tokens and hidden reasoning are prohibited in AMCX memory without exception. | Run valid, duplicate, conflicting, oversized, expired, cross-scope and secret-bearing candidates through both profiles. Results must be deterministic and secrets must never enter canonical events, quarantine payloads, logs or projections. | `merge` |
| D-0022 | `equivalent_rename` for retrieval stages, with corrected boundaries. AMX L302–315 and ECM L290–299 align on classify/filter/rank/evidence/influence. AMX supplies memory semantics; ECM supplies context assembly; identity/authorization/evidence freshness come from external owners. | Cross-product query type with tenant/project/path/task/role/time/quarantine/deletion/evidence states. Both implementations must return the same eligible ID set before ranking and the same abstention outcome. | `merge` |
| D-0026 | `missing` in AMX as a complete contract. ECM’s ArtifactRef/EvidenceRef are conceptual only (ECM L305–327); ArtifactStore owns bytes and EvidenceStore/VerificationFabric owns verdict/freshness. AMX references them but cannot assert evidence truth. | Substitute artifact bytes, subject digest, toolchain, environment or issuer. Verification must become stale/rejected without modifying AMX memory history. | `keep` |
| D-0029 | Corrected from unqualified `complementary` to `conflict` until projection semantics are fixed. AMX owns memory event persistence; ECM owns collaboration event persistence. ECM L443–454 defines an effect lifecycle even though ECM L586 assigns effects/receipts to ForgeCore. That internal dual-ownership ambiguity is unsafe. | Drive the same effect through ECM and ForgeCore with reordered, duplicated and missing events. Only ForgeCore may transition authoritative effect state; ECM must converge as a read-only projection. | `defer` |
| D-0031 | `complementary` but incomplete. AMX L439–472 covers memory poisoning, leakage and recovery. ECM L471–506 covers multi-agent/effect/circuit failures. Recovery must be severity-scoped; low-severity retrieval faults must not trigger credential rotation, while authority/secret compromise must. | Execute the same incident at informational, low, medium, high and critical severity. Required actions, write freezes, revocations, purges, rotations and re-enable evidence must follow a deterministic severity policy. | `defer` |
| D-0032 | `complementary` but not conformance-ready. AMX L330–408 and ECM L508–576 define adapter surfaces and authority constraints, but neither completes delivery, acknowledgement, resume, idempotency, cancellation-race or downgrade behavior. | Fault-inject before/after send, receive, ack, cancellation, provider compaction and resume. Each operation must produce one logical transition or a declared recoverable terminal state without weakening safety fields. | `defer` |
| D-0033 | `missing` ECM migration/exit semantics remain. AMX L584–592 defines bundle/provider exit but purge authority belongs to the deletion coordinator and schema activation to the Neutral Contract Registry. | Export, verify, replace provider, rebuild, then delete the old projection under partial failure. Canonical IDs/history must survive and deleted content must not resurrect. | `defer` |
| D-0034 | `complementary` sequencing, corrected for schema authority and acceptance. AMX L556–618 and ECM L593–631 both start from neutral contracts/conformance. ECM may evaluate schema candidates; only the reviewed Neutral Contract Registry publishes/activates them. A machine-readable task `AcceptanceContract` is absent. | Attempt schema activation from an agent, ECM promotion, provider adapter and reviewed registry; only the registry path may succeed. Task completion must fail when any machine-readable acceptance predicate lacks current evidence. | `defer` |

## 5. D-0010 canonical-memory design alternatives

| Option | Canonical shape | Strengths | Failure surface | Falsifiable decision test |
|---|---|---|---|---|
| 1. AMX + noncanonical `ECMMemoryBinding` | AMX record/event/bundle remains canonical. A separate binding references AMX record/event digests plus ECM run/task/attempt/principal/promotion/context identifiers. | Strongest ownership separation; operational churn does not alter AMX digests; binding can be projected or rebuilt. | Binding loss can reduce collaboration traceability; export must disclose whether bindings are included; atomic reference publication is required. | Delete/rebuild bindings, migrate AMX bundles without them, and replay ECM events. AMX bytes/heads remain identical; rebuilt bindings match or report explicit unresolved references. |
| 2. ECM fields as AMX extensions | ECM operational fields live in namespaced AMX extensions. | Single portable object; uses AMX unknown-extension preservation. | Operational/retention churn changes canonical event bytes; unknown critical extensions create downgrade hazards; provider-specific state contaminates canonical memory. | Change only ECM run/lease/promotion metadata. If AMX event identity changes despite unchanged memory meaning, the option fails the stability objective. |
| 3. Third neutral superset | A new schema canonically subsumes AMX and ECM memory fields. | Could remove asymmetric profiles if proven lossless. | Creates a third authority, two migrations, new legal grammar and largest conformance surface; risks demoting approved AMX semantics. | Produce lossless bidirectional mappings and independent canonicalization vectors for every AMX/ECM fixture. Any non-bijective mapping or dual owner rejects the option. |

Recommendation: **Option 1 (`keep`)** is the current least-risk hypothesis. It is not accepted merely by this document; it must pass the tests above. Option 2 is `defer`; Option 3 is `reject` unless Option 1 is falsified and a bijective superset is demonstrated.

## 6. D-0017 unknown critical extensions and downgrade

Corrected rule proposal:

1. Unknown extension bytes must be preserved byte-for-byte in canonical storage and interchange.
2. An extension declaration must state namespace, version, criticality, affected semantics, required adapter capability and downgrade behavior.
3. Unknown noncritical extensions may pass through and may be ignored semantically.
4. Unknown critical extensions must remain opaque and quarantined; semantic use, state transition, promotion, context injection and consequential projection fail closed.
5. Adapters must disclose unsupported critical capabilities and must never strip, reinterpret or silently downgrade them.

Falsification: route fixtures with unknown noncritical and critical extensions through every adapter and version downgrade. Byte equality must hold. Only the noncritical fixture may remain usable; the critical fixture must be preserved but semantically inert until a reviewed registry version activates support.

Disposition: `merge` AMX preservation with ECM fail-closed use under Neutral Contract Registry ownership.

## 7. D-0019 GateProfiles, verifier relationships, and hierarchical budgets

### 7.1 Typed subject-specific GateProfiles

Each gate decision must select a versioned profile by `subject_type` and `risk_class`:

| Subject type | Required profile components |
|---|---|
| AMX memory contract/schema | Canonicalization, round trip, causal history, extension, migration and deletion-safety gates; schema activation remains Registry-owned. |
| Memory record/promotion | Scope, provenance, EvidenceStore verdict, external memory-governance authorization, retention and cross-project approval gates. |
| Retrieval/ranking policy | Forbidden-retrieval, abstention, contradiction, decision-quality, latency, token and cost gates. |
| Prompt/skill/router configuration | Exact candidate digest, normalized baselines, hidden suites, canary, rollback, authority and unsafe-action gates. |
| Harness/adapter | Delivery, acknowledgement, replay, cancellation, downgrade, capability-disclosure and authority-invariance gates. |
| ForgeCore/effect policy | Authorization, idempotency, reconciliation, compensation and receipt gates; ECM cannot decide these transitions. |

Every GateProfile must bind the exact subject digest, baseline, suites, environment, model/provider/profile, repository revision, policy versions, metrics, decision rule, stopping rule, expiry and rollback evidence.

### 7.2 Verifier relationship predicates

A verifier is eligible only when deterministic predicates hold for the subject’s risk class:

- verifier principal differs from proposer and promotion decider where required;
- no shared mutable attempt or inherited private scratch state;
- declared model/provider/prompt/context/tool/evaluator lineage is recorded;
- correlated evidence is not counted as independent;
- verifier has no authority to mutate the subject, gate, suite, audit retention or decision threshold;
- insufficient independence requires deterministic executable or explicitly authorized human verification;
- relationship evaluation itself is versioned and auditable.

### 7.3 Hierarchical reserve-on-spawn budgets

- The ECM orchestrator owns an atomic budget ledger with run-root, task-subtree, attempt and worker reservations.
- Before spawning, the parent reserves the child’s maximum tokens, calls, wall time, monetary cost and concurrency slot from its remaining subtree budget.
- A child may subdivide only its reservation; descendant reservations cannot exceed any ancestor cap.
- Unused reservation returns atomically; consumed usage never does.
- Concurrent reservations, retries and replans count against aggregate caps; optimistic oversubscription is prohibited.
- Effect/capability limits remain ForgeCore-owned and are never inferred from compute budget.
- Exhaustion produces a typed stopped/recovery state, not silent truncation or unbounded retry.

Current constants—AMX’s three-percentage-point target; ECM’s 10,000 episodes, three rounds, depth 4, fan-out 8, three non-progress states and five denials—are **provisional** until calibrated by subject/risk class with power, confidence/credible interval, sequential-stopping and multiple-comparison rules.

Falsification: execute the maximum legal agent tree under concurrent spawn and retry. No ancestor cap may be exceeded. Then evaluate deliberately correlated verifiers and candidates near both efficacy thresholds; two independent policy evaluators must produce the same typed decision.

Disposition: `merge` as typed GateProfiles; numerical activation is `defer` pending calibration.

## 8. Corrected canonical ownership

| State-bearing domain | Canonical owner | AMX/ECM relationship |
|---|---|---|
| Memory representation, legal transition grammar, event history, causal heads and bundles | AMX | ECM holds references or noncanonical projections/bindings. |
| Principal/tenant/project/repository identity binding | Host identity and repository-identity authority | AMX and ECM record receiver-bound references only. |
| Evidence verdicts and freshness | EvidenceStore/VerificationFabric | AMX/ECM store immutable references and observed status. |
| Cross-project approval, quarantine release, trust change and visibility widening | External memory-governance policy plus host approval service | AMX records authorized transition events; ECM may request/observe. |
| Purge authorization, purge jobs, anti-resurrection barriers and deletion receipts | External deletion coordinator | AMX supplies affected memory IDs/history; stores return component receipts. |
| Collaboration tasks, attempts, roles, messages, context views and compute-budget reservations | ECM workflow log/orchestrator | External domains appear only as references/projections. |
| Durable plan/step/replan lifecycle | Typed ExecPlan | ECM tasks reference ExecPlan coordinates; reducer is not yet defined. |
| Authorization, capabilities, effects and effect receipts | ForgeCore | ECM effect state is a projection; AMX never grants authority. |
| Prompt/skill/router candidate evaluation, canary and rollback | ECM promotion service | Activation authority remains subject-specific and external where stated. |
| Schema publication and activation | Reviewed Neutral Contract Registry | ECM may evaluate candidates; AMX defines its canonical schema family but cannot self-activate a revision. |
| Artifact bytes | Content-addressed ArtifactStore | AMX/ECM reference digests. |
| Source code/configuration/history | Git at an identified revision | AMX/ECM cite; neither replaces. |
| Provider sessions | Provider adapter, opaque and noncanonical | Recovery uses ECM events/handles and canonical external state. |
| Search indexes, embeddings, summaries and caches | Derived projection owners | Rebuildable; never authoritative. |
| Toolset learning before explicit migration | `memory/toolsets/patterns.jsonl` | AMX/ECM may index or reference. |
| Telemetry and audit projections | Append-only/tamper-evident observability service | Cannot grant authority or rewrite canonical state. |

## 9. Critical blockers before synthesis or implementation

1. Complete AMX schemas and the selected `ECMMemoryBinding` profile do not exist.
2. AMX-to-ECM atomic reference publication/acknowledgement is undefined.
3. ExecPlan-to-ECM task cardinality, cancellation and completion reduction are undefined.
4. ECM/ForgeCore effect dual ownership is unresolved; ECM effect states must be specified as projections.
5. Evidence freshness and verification authority integration is undefined.
6. Raw secrets, tokens, credentials and hidden reasoning must be prohibited from all AMCX memory paths, including quarantine, logs and dead-letter storage.
7. Adapter delivery, acknowledgement, resume, idempotency, cancellation and degradation semantics are incomplete.
8. Repository/worktree/path and effect-target/argument canonicalization lacks cross-language vectors.
9. Schema publication/activation authority and Registry review protocol are incomplete.
10. A machine-readable task `AcceptanceContract` with evidence predicates, freshness, independence and completion reduction is absent.
11. Severity-scoped incident recovery and deterministic action mapping are absent.
12. Purge partial failure, anti-resurrection barriers, deletion receipt schema and restore/import behavior are undefined.
13. Cross-project approval identity, scope, validity, revocation and evidence are undefined.
14. Receiver-bound origin, external trust authority and quarantine-release transition guards are incomplete.
15. Extension criticality and adapter capability/downgrade profiles are absent.
16. Context replay binding across identity, scope, task, attempt, role/capability leases, expiry, evidence and deletion is incomplete.
17. Subject-specific GateProfile schemas, verifier-relationship predicates and deterministic decision algorithms are absent.
18. Hierarchical reserve-on-spawn budget accounting and recovery states are absent.
19. All current numeric constants remain uncalibrated by subject/risk class.
20. Task, role, memory, promotion, effect, recovery and deletion domains lack total legal transition tables.
21. Both normalization envelopes still need non-normative source-span, quote-digest, modality, extraction and alias/refinement metadata before automated compliance scoring.
22. AMX-R-0162 remains a reserved/unexplained serial and AMX-R-0203 remains an imprecise normalization row.

## 10. Mechanically derived per-requirement coverage and traceability

Quotation digest rule: `SHA-256` over the exact UTF-8 source bytes from the inclusive line span, preserving source newlines. A span may intentionally cover a complete numbered list/table row group when the normalized requirement aggregates that group. `primary_record` is valid only if the same ID appears explicitly in that record’s §3 reference list. `related_records` records all other explicit references. No ID is inferred from a numeric range.

### 10.1 AMX coverage ledger

| Requirement | Exact source span | Quotation SHA-256 | Original modality | Extraction kind | Primary record | Related records | Alias/refinement |
|---|---|---|---|---|---|---|---|
| AMX-R-0001 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0036 | none |
| AMX-R-0002 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0005, D-0036 | none |
| AMX-R-0003 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0005, D-0014, D-0036 | none |
| AMX-R-0004 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0006, D-0036 | none |
| AMX-R-0005 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0036 | none |
| AMX-R-0006 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0003, D-0036 | none |
| AMX-R-0007 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0036 | none |
| AMX-R-0008 | AMX:L37–L48 | `d2c952bfc1bc756817cea22241806826b4757f6651990a93fd0885f83bd2fedd` | explicit_must | source_normative | D-0001 | D-0004, D-0036 | none |
| AMX-R-0009 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0002 | D-0036 | none |
| AMX-R-0010 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0002 | D-0036 | none |
| AMX-R-0011 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0002 | D-0036 | none |
| AMX-R-0012 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0002 | D-0003, D-0036 | none |
| AMX-R-0013 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0004 | D-0036, D-0002 | none |
| AMX-R-0014 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0016 | D-0036 | none |
| AMX-R-0015 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0025 | D-0036 | none |
| AMX-R-0016 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0014 | D-0036 | none |
| AMX-R-0017 | AMX:L50–L63 | `34dac363189be73d35737e0d4d2e103c4c07d0863fc5dc1e15fde8fac3ca036e` | explicit_non_goal_prohibition | source_normative | D-0008 | D-0012, D-0036 | none |
| AMX-R-0018 | AMX:L3–L11 | `918f232c29e88c1279eea6d82f7f91f111bfd38f78c4bd8aaee792fa6da41ddb` | explicit_must | source_normative | D-0034 | D-0036 | none |
| AMX-R-0019 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0003 | D-0036 | none |
| AMX-R-0020 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0007 | D-0036 | none |
| AMX-R-0021 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0005 | D-0018, D-0036 | none |
| AMX-R-0022 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0015 | D-0036 | none |
| AMX-R-0023 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0003 | D-0036 | none |
| AMX-R-0024 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0006 | D-0028, D-0036 | none |
| AMX-R-0025 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0022 | D-0036 | none |
| AMX-R-0026 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0009 | D-0036 | none |
| AMX-R-0027 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0021 | D-0036 | none |
| AMX-R-0028 | AMX:L65–L76 | `34cbd902b278fc249e48c5d9dce56b0f3d97aa5568f51ebdf950f0da7c3bb5f3` | named_invariant | source_normative | D-0013 | D-0021, D-0036 | none |
| AMX-R-0029 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0015 | D-0031, D-0036 | none |
| AMX-R-0030 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0031 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0032 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0015, D-0036 | none |
| AMX-R-0033 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0015, D-0028, D-0036 | none |
| AMX-R-0034 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0005, D-0018, D-0036 | none |
| AMX-R-0035 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0036 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0015, D-0036 | none |
| AMX-R-0037 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0007, D-0036 | none |
| AMX-R-0038 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0039 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0040 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0041 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0042 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0036 | none |
| AMX-R-0043 | AMX:L439–L457 | `2432b9f99fc6bbc7160da175915eb0a90ecd490408f3f53beb3aff9a72bf0e4d` | threat_control_table | derived_design_obligation | D-0031 | D-0030, D-0036 | none |
| AMX-R-0044 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | declarative_design | derived_design_obligation | D-0020 | D-0009, D-0036 | none |
| AMX-R-0045 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0036 | none |
| AMX-R-0046 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0005, D-0009, D-0018, D-0036 | none |
| AMX-R-0047 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0036 | none |
| AMX-R-0048 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0007, D-0009, D-0036 | none |
| AMX-R-0049 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0007, D-0009, D-0026, D-0036 | none |
| AMX-R-0050 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0015, D-0036 | none |
| AMX-R-0051 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0015, D-0036 | none |
| AMX-R-0052 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0036 | none |
| AMX-R-0053 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0036 | none |
| AMX-R-0054 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0006, D-0009, D-0036 | none |
| AMX-R-0055 | AMX:L116–L132 | `b31102892fcd2a9bd576beb61a4608902f460c37d9deeec9d92e4c8af1607154` | required_axis | derived_design_obligation | D-0020 | D-0009, D-0030, D-0036 | none |
| AMX-R-0056 | AMX:L273–L283 | `470a612577ed23f6953e7704fb194fb99f56a13166d6ec9b7ed0b47333a7b31a` | imperative | derived_design_obligation | D-0009 | D-0036 | none |
| AMX-R-0057 | AMX:L273–L283 | `470a612577ed23f6953e7704fb194fb99f56a13166d6ec9b7ed0b47333a7b31a` | imperative | derived_design_obligation | D-0009 | D-0036 | none |
| AMX-R-0058 | AMX:L273–L283 | `470a612577ed23f6953e7704fb194fb99f56a13166d6ec9b7ed0b47333a7b31a` | imperative | derived_design_obligation | D-0009 | D-0036 | none |
| AMX-R-0059 | AMX:L273–L283 | `470a612577ed23f6953e7704fb194fb99f56a13166d6ec9b7ed0b47333a7b31a` | imperative | derived_design_obligation | D-0009 | D-0016, D-0036 | none |
| AMX-R-0060 | AMX:L273–L283 | `470a612577ed23f6953e7704fb194fb99f56a13166d6ec9b7ed0b47333a7b31a` | imperative | derived_design_obligation | D-0009 | D-0036 | none |
| AMX-R-0061 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0025, D-0036 | none |
| AMX-R-0062 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0063 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0064 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0012, D-0036 | none |
| AMX-R-0065 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0025, D-0036 | none |
| AMX-R-0066 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0067 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0068 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0011, D-0036 | none |
| AMX-R-0069 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0070 | AMX:L78–L91 | `a769ae139a28647087227bf72024e68914062fefcaa71912ffdfa71de33fe5b4` | source_owner_table | derived_design_obligation | D-0008 | D-0036 | none |
| AMX-R-0071 | AMX:L134–L142 | `919ff55956a0d67d30bef91314e257d872bf4f3d8e20382b4a1e1fb6e9eeebc8` | declarative_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0072 | AMX:L134–L142 | `919ff55956a0d67d30bef91314e257d872bf4f3d8e20382b4a1e1fb6e9eeebc8` | declarative_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0073 | AMX:L134–L142 | `919ff55956a0d67d30bef91314e257d872bf4f3d8e20382b4a1e1fb6e9eeebc8` | explicit_must | derived_design_obligation | D-0010 | D-0017, D-0036, D-0038 | none |
| AMX-R-0074 | AMX:L144–L230 | `f0bc5e117c3bdafddc4222acb7be9c818d590c4c911dbdc40644cdb09dea65a9` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0075 | AMX:L144–L230 | `f0bc5e117c3bdafddc4222acb7be9c818d590c4c911dbdc40644cdb09dea65a9` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0076 | AMX:L144–L230 | `f0bc5e117c3bdafddc4222acb7be9c818d590c4c911dbdc40644cdb09dea65a9` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0077 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0078 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0079 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0009, D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0080 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0009, D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0081 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0009, D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0082 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0009, D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0083 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0009, D-0011, D-0013, D-0036, D-0038 | none |
| AMX-R-0084 | AMX:L233–L255 | `6d3413317e4c3f17301cfc900b34ec28885f6500615d6664350943cffdeb9b92` | declared_required_contract | derived_design_obligation | D-0010 | D-0011, D-0036, D-0038 | none |
| AMX-R-0085 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0086 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0087 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0088 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0089 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0026, D-0036, D-0038 | none |
| AMX-R-0090 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0026, D-0036, D-0038 | none |
| AMX-R-0091 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0092 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0093 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0094 | AMX:L257–L271 | `acd34f16c5191e8e5b52f795a4479ad13fa4e0d09cb0653dc910336b19ba8f19` | declared_required_contract | derived_design_obligation | D-0010 | D-0036, D-0038 | none |
| AMX-R-0095 | AMX:L93–L102 | `ce905bf6fa4f440c02a01ebca7e0416000751103420a599ff321a0f8b758e609` | declarative_lifecycle | derived_design_obligation | D-0011 | D-0029, D-0036 | none |
| AMX-R-0096 | AMX:L93–L102 | `ce905bf6fa4f440c02a01ebca7e0416000751103420a599ff321a0f8b758e609` | declarative_lifecycle | derived_design_obligation | D-0016 | D-0036 | none |
| AMX-R-0097 | AMX:L93–L102 | `ce905bf6fa4f440c02a01ebca7e0416000751103420a599ff321a0f8b758e609` | declarative_lifecycle | derived_design_obligation | D-0016 | D-0036 | none |
| AMX-R-0098 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0015 | D-0013, D-0021, D-0036 | none |
| AMX-R-0099 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0015 | D-0013, D-0021, D-0036 | none |
| AMX-R-0100 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0021 | D-0013, D-0036, D-0039 | none |
| AMX-R-0101 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0021 | D-0013, D-0036 | none |
| AMX-R-0102 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0021 | D-0013, D-0036 | none |
| AMX-R-0103 | AMX:L285–L300 | `1f2156a67be5c7de69c3269a6ab6d0c2e09774ab38d644626771fea322206cd8` | imperative_or_constraint | derived_design_obligation | D-0021 | D-0013, D-0036 | none |
| AMX-R-0104 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0011 | D-0006, D-0029, D-0036 | none |
| AMX-R-0105 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0011 | D-0006, D-0029, D-0036 | none |
| AMX-R-0106 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0011 | D-0006, D-0029, D-0036 | none |
| AMX-R-0107 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0011 | D-0006, D-0029, D-0036 | none |
| AMX-R-0108 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0016 | D-0029, D-0036 | none |
| AMX-R-0109 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | imperative_or_constraint | derived_design_obligation | D-0016 | D-0029, D-0036 | none |
| AMX-R-0110 | AMX:L459–L472 | `db7a8404c47b8def78536f42526023786e17b8c0fe694ae6d2d44eb15fa43746` | imperative_or_constraint | derived_design_obligation | D-0031 | D-0016, D-0036 | none |
| AMX-R-0111 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0036 | refines AMX-R-0242 |
| AMX-R-0112 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0036 | refines AMX-R-0242 |
| AMX-R-0113 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0036 | refines AMX-R-0242 |
| AMX-R-0114 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0036 | refines AMX-R-0242 |
| AMX-R-0115 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0013, D-0014, D-0036 | refines AMX-R-0242 |
| AMX-R-0116 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | rollout_directive | derived_design_obligation | D-0034 | D-0036 | refines AMX-R-0242 |
| AMX-R-0117 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0118 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0119 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0120 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0121 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0122 | AMX:L104–L114 | `3592d0950acc16ff507c7ab5f04978bd182f33085c16c383c79f80485f3e6339` | actor_permission_table | derived_design_obligation | D-0003 | D-0036 | none |
| AMX-R-0123 | AMX:L302–L315 | `f7d238585c9050b1f1d2457394f97f40e781e30cdb16709c1d9c7869d3090cc7` | declarative_design | derived_design_obligation | D-0003 | D-0022, D-0036 | none |
| AMX-R-0124 | AMX:L410–L420 | `9cf7f91cfeee3edd32a6e97764025f9496042f8795ca2245cc862abba89720f2` | declarative_design | derived_design_obligation | D-0025 | D-0036 | none |
| AMX-R-0125 | AMX:L410–L420 | `9cf7f91cfeee3edd32a6e97764025f9496042f8795ca2245cc862abba89720f2` | declarative_design | derived_design_obligation | D-0025 | D-0036 | none |
| AMX-R-0126 | AMX:L410–L420 | `9cf7f91cfeee3edd32a6e97764025f9496042f8795ca2245cc862abba89720f2` | declarative_design | derived_design_obligation | D-0025 | D-0036 | none |
| AMX-R-0127 | AMX:L93–L102 | `ce905bf6fa4f440c02a01ebca7e0416000751103420a599ff321a0f8b758e609` | declarative_design | derived_design_obligation | D-0029 | D-0008, D-0036 | none |
| AMX-R-0128 | AMX:L93–L102 | `ce905bf6fa4f440c02a01ebca7e0416000751103420a599ff321a0f8b758e609` | declarative_design | derived_design_obligation | D-0029 | D-0008, D-0036 | none |
| AMX-R-0129 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | declarative_design | derived_design_obligation | D-0029 | D-0032, D-0036 | none |
| AMX-R-0130 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | declarative_design | derived_design_obligation | D-0029 | D-0032, D-0036 | none |
| AMX-R-0131 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | declarative_design | derived_design_obligation | D-0029 | D-0032, D-0036 | none |
| AMX-R-0132 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | declarative_design | derived_design_obligation | D-0029 | D-0032, D-0036 | none |
| AMX-R-0133 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | declarative_design | derived_design_obligation | D-0029 | D-0032, D-0036 | none |
| AMX-R-0134 | AMX:L348–L366 | `cdd0d56ca1bee61ead43aeb7f8b9d9e8f7651f2fc9aefa7e6f05c43dc3174714` | declarative_design | derived_design_obligation | D-0029 | D-0025, D-0036 | none |
| AMX-R-0135 | AMX:L348–L366 | `cdd0d56ca1bee61ead43aeb7f8b9d9e8f7651f2fc9aefa7e6f05c43dc3174714` | declarative_design | derived_design_obligation | D-0029 | D-0025, D-0036 | none |
| AMX-R-0136 | AMX:L348–L366 | `cdd0d56ca1bee61ead43aeb7f8b9d9e8f7651f2fc9aefa7e6f05c43dc3174714` | declarative_design | derived_design_obligation | D-0029 | D-0025, D-0036 | none |
| AMX-R-0137 | AMX:L348–L366 | `cdd0d56ca1bee61ead43aeb7f8b9d9e8f7651f2fc9aefa7e6f05c43dc3174714` | declarative_design | derived_design_obligation | D-0029 | D-0025, D-0036 | none |
| AMX-R-0138 | AMX:L368–L372 | `9f4dfea771ec1dbf1703327f16b11033564a205161fa0cea79d2a7c4218dc38f` | declarative_design | derived_design_obligation | D-0029 | D-0036 | none |
| AMX-R-0139 | AMX:L368–L372 | `9f4dfea771ec1dbf1703327f16b11033564a205161fa0cea79d2a7c4218dc38f` | declarative_design | derived_design_obligation | D-0029 | D-0011, D-0036 | none |
| AMX-R-0140 | AMX:L317–L328 | `b393d2db356fb4935bf620e9b8d5134d46f79460e4b963bdf93ffa293c4ea461` | declarative_design | derived_design_obligation | D-0029 | D-0036 | none |
| AMX-R-0141 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0142 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0143 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0144 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0145 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0146 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0147 | AMX:L374–L397 | `7b8cfd6b1d911521f2940ef91f27690f07007cfa6209801df6370850b606dcb6` | declared_interface | derived_design_obligation | D-0023 | D-0036 | none |
| AMX-R-0148 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0149 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0150 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0151 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0152 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0153 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0154 | AMX:L399–L408 | `7f041e437ab4c9b4f505489ed2da2369ed33dd34e1e7cc876e3b9012ce5d2312` | declared_interface | derived_design_obligation | D-0024 | D-0036 | none |
| AMX-R-0155 | AMX:L13–L23 | `b9a40dee126d0aabd3686baf8c2f46013a7252f62a44599f537e52be9c1ae565` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0036 | none |
| AMX-R-0156 | AMX:L13–L23 | `b9a40dee126d0aabd3686baf8c2f46013a7252f62a44599f537e52be9c1ae565` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0036 | none |
| AMX-R-0157 | AMX:L422–L437 | `4d66aa9f759a3306414aceeb2120c41f6d90dfc76a52870b5e21eecea1dd0ff1` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0036 | none |
| AMX-R-0158 | AMX:L422–L437 | `4d66aa9f759a3306414aceeb2120c41f6d90dfc76a52870b5e21eecea1dd0ff1` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0035, D-0036, D-0039 | none |
| AMX-R-0159 | AMX:L422–L437 | `4d66aa9f759a3306414aceeb2120c41f6d90dfc76a52870b5e21eecea1dd0ff1` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0035, D-0036 | none |
| AMX-R-0160 | AMX:L594–L618 | `98150bde6a6d31aa20a2f51dfc8f718c0ca5022a333b510f9e7eb99836f09ce4` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0017, D-0036 | none |
| AMX-R-0161 | AMX:L332–L346 | `fc3a1573492d29064872fcccdf50df7fbab8d587aae572397f8d3590195717b5` | explicit_or_declarative_constraint | derived_design_obligation | D-0032 | D-0036 | none |
| AMX-R-0163 | AMX:L410–L420 | `9cf7f91cfeee3edd32a6e97764025f9496042f8795ca2245cc862abba89720f2` | declarative_generation_rule | derived_design_obligation | D-0025 | D-0035, D-0036 | none |
| AMX-R-0164 | AMX:L302–L315 | `f7d238585c9050b1f1d2457394f97f40e781e30cdb16709c1d9c7869d3090cc7` | evaluation_directive | derived_design_obligation | D-0022 | D-0018, D-0036, D-0039 | none |
| AMX-R-0165 | AMX:L474–L503 | `796d17da607ca37f8ed4ef50d9b6d1be5295bcccfd1581331aacac6a4ef9e313` | evaluation_directive | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0166 | AMX:L474–L503 | `796d17da607ca37f8ed4ef50d9b6d1be5295bcccfd1581331aacac6a4ef9e313` | evaluation_directive | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0167 | AMX:L474–L503 | `796d17da607ca37f8ed4ef50d9b6d1be5295bcccfd1581331aacac6a4ef9e313` | evaluation_directive | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0168 | AMX:L474–L503 | `796d17da607ca37f8ed4ef50d9b6d1be5295bcccfd1581331aacac6a4ef9e313` | evaluation_directive | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0169 | AMX:L505–L536 | `6ff664ab235b6a001abf65a22e3c8eb9014f4658085682d42145a505d4b912e0` | evaluation_directive | acceptance_or_evaluation_obligation | D-0030 | D-0019, D-0036 | none |
| AMX-R-0170 | AMX:L505–L536 | `6ff664ab235b6a001abf65a22e3c8eb9014f4658085682d42145a505d4b912e0` | evaluation_directive | acceptance_or_evaluation_obligation | D-0030 | D-0019, D-0036 | none |
| AMX-R-0171 | AMX:L505–L536 | `6ff664ab235b6a001abf65a22e3c8eb9014f4658085682d42145a505d4b912e0` | evaluation_directive | acceptance_or_evaluation_obligation | D-0030 | D-0019, D-0036 | none |
| AMX-R-0172 | AMX:L505–L536 | `6ff664ab235b6a001abf65a22e3c8eb9014f4658085682d42145a505d4b912e0` | evaluation_directive | acceptance_or_evaluation_obligation | D-0030 | D-0019, D-0036 | none |
| AMX-R-0173 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0174 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0175 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0018, D-0036 | none |
| AMX-R-0176 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0177 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0178 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0007, D-0036 | none |
| AMX-R-0179 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036 | none |
| AMX-R-0180 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036, D-0040 | none |
| AMX-R-0181 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0022, D-0036 | none |
| AMX-R-0182 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036, D-0040 | none |
| AMX-R-0183 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0016, D-0036 | none |
| AMX-R-0184 | AMX:L538–L554 | `25247e1f47b0457b57fae9819f7d204cdc23c631ba331c7840c6c768c0dd89e3` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0036, D-0040 | none |
| AMX-R-0185 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_gate | acceptance_or_evaluation_obligation | D-0019 | D-0034, D-0036 | none |
| AMX-R-0186 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0187 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0188 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0189 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0190 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0191 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0192 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0036 | none |
| AMX-R-0193 | AMX:L584–L592 | `c36fba86910e4a50985168b6b1b7bbcb51141845478c75c1266933cce297579e` | migration_directive | migration_obligation | D-0033 | D-0016, D-0017, D-0036 | none |
| AMX-R-0194 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0195 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0196 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0197 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0198 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0199 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0200 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0201 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0202 | AMX:L556–L571 | `34acda14196307d1ce1478868904c2e6d152b82b0a0d12e02363d7d77c5026da` | implementation_order | implementation_order | D-0034 | D-0036 | none |
| AMX-R-0203 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036, D-0038 | none |
| AMX-R-0204 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036, D-0038 | none |
| AMX-R-0205 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036, D-0038 | none |
| AMX-R-0206 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0017, D-0036, D-0038 | refines AMX-R-0160 |
| AMX-R-0207 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036 | none |
| AMX-R-0208 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036 | refines AMX-R-0114 |
| AMX-R-0209 | AMX:L605–L618 | `4afd625b6bf61893ac1a3d2fc589b8d4c02d9b5875f04c2813faf8d4b56b8928` | acceptance_condition | acceptance_obligation | D-0034 | D-0036 | none |
| AMX-R-0210 | AMX:L287–L287 | `7cc0a8591a4558fd50c54a4cbe9a1605f1bd59407965b49ef9530038cece46cb` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100 |
| AMX-R-0211 | AMX:L288–L288 | `9e3ec188ee9dacc0b0433d71b8a5fb233c0ac71addc0655477f7d9e52eb23b4b` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100 |
| AMX-R-0212 | AMX:L289–L289 | `313af7883ac8eb4a6193d7eb5bfe1c7b8803a7d43a5b950eb1546ea7a8ade110` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100 |
| AMX-R-0213 | AMX:L290–L290 | `16b07cd56e47f7bd657299bc6faa59229cfbd75b3dfb1fd3318c3d8ca21b706b` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100/0011/0039 |
| AMX-R-0214 | AMX:L291–L291 | `a3ac4702da7570c29196c157af964e75b2a7dc8b9534ed8101adc4915cf053bb` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100 |
| AMX-R-0215 | AMX:L292–L292 | `0424a2471468923647213b6bd5579e0cc30657d1744cb928c836a39cd0105dc6` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0015, D-0036, D-0039 | refines AMX-R-0100/0022/0036 |
| AMX-R-0216 | AMX:L293–L293 | `73f167880a054a73afec9b2fec43f32d6fef0d9ba79388eb3c2adb210f29bc16` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0015, D-0036, D-0039 | refines AMX-R-0100/0098 |
| AMX-R-0217 | AMX:L294–L294 | `6053497d6b9583c7cc4992761c05294eedcc38183711a13491bf5ca17c0da07d` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100/0054 |
| AMX-R-0218 | AMX:L295–L295 | `dcd3fbf94ab5ad0523beabe96b1ae46a72e3b80cc434a7658431006fdd300de3` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100/0054 |
| AMX-R-0219 | AMX:L296–L296 | `e9a2409bcbbaf08afd9b382d057dfe24741d45c8f550e0c0eba4b772e8701994` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100/0028/0101/0102 |
| AMX-R-0220 | AMX:L297–L297 | `aa2efd2f7822f8928e2bd64fe28a1dc8f6aa26f3cc225cfc56a06eedde562b3f` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | alias_of AMX-R-0103; refines AMX-R-0100 |
| AMX-R-0221 | AMX:L298–L298 | `266f055ffe86f0ab5058d48f927ce12c0f0b548a501d387066362c7450d72dee` | atomic_refinement_of_source_directive | normalization_refinement | D-0021 | D-0036, D-0039 | refines AMX-R-0100 |
| AMX-R-0222 | AMX:L304–L304 | `3c7e33f09823074f43505baee32fee9bf673c4b195499ab58d2932777ad9ae8a` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164 |
| AMX-R-0223 | AMX:L305–L305 | `3c2073dc021fbd880746a9346506157e0be23772f6efddcce2712c41617d5c6f` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164/0021 |
| AMX-R-0224 | AMX:L306–L306 | `47274ddd2ef293bfe89688108751f3ae7a890b62860d4b7af10fd6032c91155c` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164 |
| AMX-R-0225 | AMX:L307–L307 | `4614c96b9d2373f13e904b0fb97d9e17998489a89c1dce8dbe3b42a2628a2a84` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164 |
| AMX-R-0226 | AMX:L308–L308 | `03f746675497253f5cfc43ee44a57664a2cd321a80311c6afaa86ca19dcd2371` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164 |
| AMX-R-0227 | AMX:L309–L309 | `0118c5cef52201c9fa3909e3601bfa77ee0a094ce2d27bf051f87f0f20d6eb7c` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | alias_of AMX-R-0181; refines AMX-R-0164 |
| AMX-R-0228 | AMX:L310–L310 | `76d0c9f90433fdc9d394734d27d2566265e47eb819b9c0fa4aa7373360acedf9` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164 |
| AMX-R-0229 | AMX:L311–L311 | `80de2665a1663ec34ab9aa5bc62f5e2af5a558e914fa351f54d1e0b8d80f7718` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164/0123 |
| AMX-R-0230 | AMX:L312–L312 | `74b5d6e2d217c03e55cdf7a85c6dc46a4dbaeff42390877236dcf9587471bba7` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164/0055 |
| AMX-R-0231 | AMX:L313–L313 | `ae03767bb16a7c409d13f60f37fd9d0d09136579cf09deb0989cfd32fe2c04b3` | atomic_refinement_of_source_directive | normalization_refinement | D-0022 | D-0018, D-0036, D-0039 | refines AMX-R-0164/0025 |
| AMX-R-0232 | AMX:L426–L426 | `bfadb9fcd9c09478f8cdb5cf1c07ba9f6ab0c1d2a001c91607a3af5e73577a95` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0233 | AMX:L427–L427 | `20d4247329dee17b2998023373c311a9c8ce8c94cbf71523be65444d25fd856e` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0234 | AMX:L428–L428 | `be1cd11b343eb6f1790778775b9c9f344a0139fed33a619408a1477007b1de75` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0235 | AMX:L429–L429 | `64d0a5cab526ce5d913d8eadeb87c4b63d6de7d3f083e7f2e427495139b1a6f6` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0236 | AMX:L430–L430 | `b3ad13fbde6248f684b50f6928cf1b10aa5c4f7ab193ada65aa462ede809eee0` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0237 | AMX:L431–L431 | `70bcccb7add7476573e059f5a3e49b7582fe250927507c2074689b5b62a60087` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0238 | AMX:L432–L432 | `7dca2c85b40ccd897c258860fb8a909c849fd2a1d11d675ca83362ffa044c7b6` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158/0011 |
| AMX-R-0239 | AMX:L433–L433 | `1c67c69e8512486c9f266f7614a90910f8625bd6fe371916f3671d87ddb4d9a3` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0014, D-0036, D-0039 | refines AMX-R-0158/0016 |
| AMX-R-0240 | AMX:L434–L434 | `e347c66caa54daff0937b514c2750c801f172a726dbbea2a1ded61aae9067ad1` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0241 | AMX:L435–L435 | `c2aaa8b00c6ec2f79025ed05d5437f7cb25ed4de052b307b03710e4d4ebe54ef` | atomic_refinement_of_source_directive | normalization_refinement | D-0035 | D-0036, D-0039 | refines AMX-R-0158 |
| AMX-R-0242 | AMX:L573–L582 | `8181cda656a8cacb40763d1bc7e5371ceadcf860dd127a7c52984799d01636a4` | atomic_refinement_of_source_directive | normalization_refinement | D-0034 | D-0036, D-0039 | none |
| AMX-R-0243 | AMX:L348–L366 | `cdd0d56ca1bee61ead43aeb7f8b9d9e8f7651f2fc9aefa7e6f05c43dc3174714` | atomic_refinement_of_source_directive | normalization_refinement | D-0034 | D-0025, D-0036, D-0039 | none |
| AMX-R-0244 | AMX:L582–L582 | `ba9f5be3b9285289fb2c0f48bbf2c1437b199ff5fd642fe6eab76ce994dba4d0` | atomic_refinement_of_source_directive | normalization_refinement | D-0034 | D-0036, D-0039 | none |

`AMX-R-0162` is not a published requirement. It remains a reserved/unexplained serial and is excluded from the 243-row denominator.

### 10.2 ECM coverage ledger

| Requirement | Exact source span | Quotation SHA-256 | Original modality | Extraction kind | Primary record | Related records | Alias/refinement |
|---|---|---|---|---|---|---|---|
| ECM-R-0001 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0002 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0005, D-0037 | none |
| ECM-R-0003 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0006, D-0037 | none |
| ECM-R-0004 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0005 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0006 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0007 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0008 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0003, D-0037 | none |
| ECM-R-0009 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0004, D-0037 | none |
| ECM-R-0010 | ECM:L19–L32 | `20c9796a75ca6ef19906978c3bec3786702848a26441167da84e263709379e0a` | explicit_must | source_normative | D-0001 | D-0037 | none |
| ECM-R-0011 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0002 | D-0037 | none |
| ECM-R-0012 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0002 | D-0037 | none |
| ECM-R-0013 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0003 | D-0037, D-0039 | refines ECM-R-0021 |
| ECM-R-0014 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0035 | D-0037 | none |
| ECM-R-0015 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0003 | D-0008, D-0037 | none |
| ECM-R-0016 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0004 | D-0037 | none |
| ECM-R-0017 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0008 | D-0037 | none |
| ECM-R-0018 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0019 | D-0004, D-0037 | none |
| ECM-R-0019 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0019 | D-0037 | none |
| ECM-R-0020 | ECM:L34–L47 | `7de11c758d34625988ef45dab757b503342f8df253af903e4ae3912bd4cffee9` | explicit_will_not_prohibition | source_normative | D-0032 | D-0003, D-0037, D-0039 | refines ECM-R-0021 |
| ECM-R-0021 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0003 | D-0037 | none |
| ECM-R-0022 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0003 | D-0015, D-0037 | none |
| ECM-R-0023 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0003 | D-0037 | none |
| ECM-R-0024 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0003 | D-0019, D-0037 | none |
| ECM-R-0025 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0003 | D-0037 | none |
| ECM-R-0026 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0005 | D-0022, D-0037 | none |
| ECM-R-0027 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0005 | D-0018, D-0037 | none |
| ECM-R-0028 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0006 | D-0037 | none |
| ECM-R-0029 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0029 | D-0037 | none |
| ECM-R-0030 | ECM:L49–L60 | `f24249472256397203d298e00aecd93223105ad2a583ad2bb5c2ebea180142d1` | named_invariant | source_normative | D-0019 | D-0014, D-0037 | none |
| ECM-R-0031 | ECM:L62–L82 | `4e284c5d2f3b13a66c37b343f6075b78e7b31be3961a579a5b868713034078a0` | recommended_topology_declarative | derived_design_obligation | D-0012 | D-0008, D-0037 | none |
| ECM-R-0032 | ECM:L62–L82 | `4e284c5d2f3b13a66c37b343f6075b78e7b31be3961a579a5b868713034078a0` | recommended_topology_declarative | derived_design_obligation | D-0012 | D-0037 | none |
| ECM-R-0033 | ECM:L62–L82 | `4e284c5d2f3b13a66c37b343f6075b78e7b31be3961a579a5b868713034078a0` | recommended_topology_declarative | derived_design_obligation | D-0012 | D-0037 | none |
| ECM-R-0034 | ECM:L62–L82 | `4e284c5d2f3b13a66c37b343f6075b78e7b31be3961a579a5b868713034078a0` | recommended_topology_declarative | derived_design_obligation | D-0012 | D-0037 | none |
| ECM-R-0035 | ECM:L62–L82 | `4e284c5d2f3b13a66c37b343f6075b78e7b31be3961a579a5b868713034078a0` | recommended_topology_declarative | derived_design_obligation | D-0012 | D-0037 | none |
| ECM-R-0036 | ECM:L86–L88 | `7b7dce3cae378a94b4e1ede96f477c2a0c647072a683e40d4d7e1c0503ff1a66` | protocol_declarative | derived_design_obligation | D-0024 | D-0009, D-0037 | none |
| ECM-R-0037 | ECM:L90–L92 | `a77ca5b26cd6cc855f7bc06f9108bd59a1815853c8be0d8d77775f4835d3e15e` | protocol_declarative | derived_design_obligation | D-0023 | D-0037 | none |
| ECM-R-0038 | ECM:L90–L92 | `a77ca5b26cd6cc855f7bc06f9108bd59a1815853c8be0d8d77775f4835d3e15e` | protocol_declarative | derived_design_obligation | D-0023 | D-0037 | none |
| ECM-R-0039 | ECM:L94–L96 | `ecbc153cf54b0145b26d9c76971d68a214d75db87abdeb4e8a872b7ab9679e2c` | protocol_declarative | derived_design_obligation | D-0011 | D-0037 | none |
| ECM-R-0040 | ECM:L94–L96 | `ecbc153cf54b0145b26d9c76971d68a214d75db87abdeb4e8a872b7ab9679e2c` | protocol_declarative | derived_design_obligation | D-0011 | D-0024, D-0037 | none |
| ECM-R-0041 | ECM:L98–L100 | `e2eba8ebb87ed113b876d130451b5a48eb97688a7bef866235125aa9bebe0229` | protocol_declarative | derived_design_obligation | D-0030 | D-0009, D-0037 | none |
| ECM-R-0042 | ECM:L98–L100 | `e2eba8ebb87ed113b876d130451b5a48eb97688a7bef866235125aa9bebe0229` | protocol_declarative | derived_design_obligation | D-0030 | D-0009, D-0037 | none |
| ECM-R-0043 | ECM:L102–L104 | `4a1e4bc4faf88e80e3a707a3ec56fe80c1b186b7036a8ec299e43e42cfb56371` | protocol_declarative | derived_design_obligation | D-0030 | D-0009, D-0037 | none |
| ECM-R-0044 | ECM:L102–L104 | `4a1e4bc4faf88e80e3a707a3ec56fe80c1b186b7036a8ec299e43e42cfb56371` | protocol_declarative | derived_design_obligation | D-0030 | D-0009, D-0037 | none |
| ECM-R-0045 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0046 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0009, D-0018, D-0037, D-0038 | none |
| ECM-R-0047 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0048 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0049 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0050 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0017, D-0037, D-0038 | none |
| ECM-R-0051 | ECM:L106–L148 | `e9dcfdab777e1e0c1cc9faca1b8fb84dcd7b6b97d1217defa31399b25dc02ecd` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0018, D-0037, D-0038 | none |
| ECM-R-0052 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0037, D-0038 | none |
| ECM-R-0053 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0037, D-0038 | none |
| ECM-R-0054 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0037, D-0038 | none |
| ECM-R-0055 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0018, D-0037, D-0038 | none |
| ECM-R-0056 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0037, D-0038 | none |
| ECM-R-0057 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0019, D-0037, D-0038, D-0039 | refines ECM-R-0024 |
| ECM-R-0058 | ECM:L150–L174 | `08abe5fbe880b36aa925837ead0e2c86ceb56e0d50137fa3f2808a711315324b` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0012, D-0019, D-0037, D-0038 | none |
| ECM-R-0059 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0060 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0061 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0062 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0005, D-0018, D-0037, D-0038 | none |
| ECM-R-0063 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0064 | ECM:L176–L209 | `bdfe211a7997697d3d787fb95896cc87c5f86e51c7d6fdfb064ef2a07379f8be` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0037, D-0038 | none |
| ECM-R-0065 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0013, D-0037, D-0038 | none |
| ECM-R-0066 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0013, D-0037, D-0038 | none |
| ECM-R-0067 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0013, D-0015, D-0021, D-0037, D-0038 | none |
| ECM-R-0068 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0005, D-0013, D-0018, D-0022, D-0037, D-0038 | none |
| ECM-R-0069 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0013, D-0015, D-0021, D-0037, D-0038 | none |
| ECM-R-0070 | ECM:L211–L225 | `16e28ec0f7ceaee9636690d3c9a16e87766d5ed68a84ab5a1363e6fa9ad1389e` | declared_contract_or_lifecycle | derived_design_obligation | D-0027 | D-0013, D-0021, D-0037, D-0038 | none |
| ECM-R-0071 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0072 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0073 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0014, D-0037, D-0038 | none |
| ECM-R-0074 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0015, D-0037, D-0038 | none |
| ECM-R-0075 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0076 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0077 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0078 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0079 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0080 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0081 | ECM:L227–L243 | `2b31626452d5cccffb799ba3e9703a14ff9e42d6f2c79c865a1e2970a358ec9f` | declared_contract_or_lifecycle | derived_design_obligation | D-0020 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0082 | ECM:L245–L276 | `305056aee74e15c92abe5ed0c90ec8c55072c7bb1c085b13c9757211062664ab` | declared_contract_or_lifecycle | derived_design_obligation | D-0010 | D-0013, D-0015, D-0037, D-0038 | none |
| ECM-R-0083 | ECM:L245–L276 | `305056aee74e15c92abe5ed0c90ec8c55072c7bb1c085b13c9757211062664ab` | declared_contract_or_lifecycle | derived_design_obligation | D-0010 | D-0013, D-0037, D-0038 | none |
| ECM-R-0084 | ECM:L245–L276 | `305056aee74e15c92abe5ed0c90ec8c55072c7bb1c085b13c9757211062664ab` | declared_contract_or_lifecycle | derived_design_obligation | D-0010 | D-0013, D-0037, D-0038 | refines ECM-R-0028 |
| ECM-R-0085 | ECM:L245–L276 | `305056aee74e15c92abe5ed0c90ec8c55072c7bb1c085b13c9757211062664ab` | declared_contract_or_lifecycle | derived_design_obligation | D-0010 | D-0013, D-0037, D-0038 | none |
| ECM-R-0086 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0015, D-0037, D-0038 | none |
| ECM-R-0087 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0015, D-0037, D-0038 | none |
| ECM-R-0088 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0089 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0006, D-0010, D-0013, D-0037, D-0038 | refines ECM-R-0028 |
| ECM-R-0090 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0091 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0092 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0016, D-0037, D-0038 | none |
| ECM-R-0093 | ECM:L278–L288 | `867033827928e391f980f0279ac961c86979850c3e046e63b3bd11baba89acc0` | imperative_path | derived_design_obligation | D-0021 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0094 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0095 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0005, D-0010, D-0013, D-0016, D-0018, D-0037, D-0038 | refines ECM-R-0026 |
| ECM-R-0096 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0097 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0098 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0099 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0100 | ECM:L290–L299 | `ee8ef03e4cf6bbd7e2b95b82ec69abb0c2cbe6478d19c46641f89699cb9e47ac` | imperative_path | derived_design_obligation | D-0022 | D-0010, D-0013, D-0037, D-0038 | none |
| ECM-R-0101 | ECM:L301–L303 | `8e8173896a77ce5afa1c516c905178587c09a86650c0a23ca14274b90e8dabb7` | declarative_or_explicit_constraint | derived_design_obligation | D-0033 | D-0010, D-0037, D-0038 | none |
| ECM-R-0102 | ECM:L301–L303 | `8e8173896a77ce5afa1c516c905178587c09a86650c0a23ca14274b90e8dabb7` | declarative_or_explicit_constraint | derived_design_obligation | D-0033 | D-0010, D-0037, D-0038 | none |
| ECM-R-0103 | ECM:L305–L327 | `371f0fe001237d3d59e803efe428e02f769ebe8a0c0876bfde027663fd453d69` | declarative_or_explicit_constraint | derived_design_obligation | D-0026 | D-0007, D-0009, D-0037, D-0038 | none |
| ECM-R-0104 | ECM:L305–L327 | `371f0fe001237d3d59e803efe428e02f769ebe8a0c0876bfde027663fd453d69` | declarative_or_explicit_constraint | derived_design_obligation | D-0026 | D-0007, D-0009, D-0037, D-0038 | none |
| ECM-R-0105 | ECM:L305–L327 | `371f0fe001237d3d59e803efe428e02f769ebe8a0c0876bfde027663fd453d69` | declarative_or_explicit_constraint | derived_design_obligation | D-0026 | D-0007, D-0037, D-0038 | none |
| ECM-R-0106 | ECM:L305–L327 | `371f0fe001237d3d59e803efe428e02f769ebe8a0c0876bfde027663fd453d69` | declarative_or_explicit_constraint | derived_design_obligation | D-0026 | D-0037, D-0038 | none |
| ECM-R-0107 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0003, D-0037 | none |
| ECM-R-0108 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0014, D-0019, D-0037 | none |
| ECM-R-0109 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0037 | none |
| ECM-R-0110 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0006, D-0037 | refines ECM-R-0028 |
| ECM-R-0111 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0006, D-0037 | none |
| ECM-R-0112 | ECM:L329–L340 | `c8865a707a8d227c1641aa8752fd1453e1f44c6de542cc4e0e6bb8c45b39c340` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0003, D-0006, D-0037 | none |
| ECM-R-0113 | ECM:L342–L362 | `19e037865ade2784ae2dda247ee845c8357fdca74a0a98c708e5effed8c31964` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0037 | none |
| ECM-R-0114 | ECM:L342–L362 | `19e037865ade2784ae2dda247ee845c8357fdca74a0a98c708e5effed8c31964` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0037 | none |
| ECM-R-0115 | ECM:L342–L362 | `19e037865ade2784ae2dda247ee845c8357fdca74a0a98c708e5effed8c31964` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0037 | none |
| ECM-R-0116 | ECM:L342–L362 | `19e037865ade2784ae2dda247ee845c8357fdca74a0a98c708e5effed8c31964` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0037 | none |
| ECM-R-0117 | ECM:L364–L378 | `a4efae642dd04c601e5e6804aa4e0133659c2a587da62f537617e335b9366677` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0013, D-0037 | none |
| ECM-R-0118 | ECM:L364–L378 | `a4efae642dd04c601e5e6804aa4e0133659c2a587da62f537617e335b9366677` | declarative_or_explicit_constraint | derived_design_obligation | D-0028 | D-0013, D-0019, D-0037 | none |
| ECM-R-0119 | ECM:L380–L390 | `2043c9a7bf6f44a5196a2e79827e7a306fd438c89f98715c1898dcf9f5ea4788` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0120 | ECM:L380–L390 | `2043c9a7bf6f44a5196a2e79827e7a306fd438c89f98715c1898dcf9f5ea4788` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0121 | ECM:L392–L401 | `e250a363de873cd4504af830a71c652946b0acbf48c917b55d503c3068477540` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0014, D-0037 | none |
| ECM-R-0122 | ECM:L392–L401 | `e250a363de873cd4504af830a71c652946b0acbf48c917b55d503c3068477540` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037, D-0040 | none |
| ECM-R-0123 | ECM:L392–L401 | `e250a363de873cd4504af830a71c652946b0acbf48c917b55d503c3068477540` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0028, D-0037, D-0039 | refines ECM-R-0024 |
| ECM-R-0124 | ECM:L392–L401 | `e250a363de873cd4504af830a71c652946b0acbf48c917b55d503c3068477540` | promotion_directive | derived_design_obligation | D-0019 | D-0009, D-0013, D-0037 | none |
| ECM-R-0125 | ECM:L403–L420 | `883513af5b66985baf16e4953175687af95d55c0bad53fada4dd4eb31ab05219` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0126 | ECM:L403–L420 | `883513af5b66985baf16e4953175687af95d55c0bad53fada4dd4eb31ab05219` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0127 | ECM:L403–L420 | `883513af5b66985baf16e4953175687af95d55c0bad53fada4dd4eb31ab05219` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0128 | ECM:L403–L420 | `883513af5b66985baf16e4953175687af95d55c0bad53fada4dd4eb31ab05219` | promotion_directive | derived_design_obligation | D-0019 | D-0013, D-0037 | none |
| ECM-R-0129 | ECM:L422–L430 | `0d56eda7a5a05c90c3d8d54e2c89b13a31889100e0a0d9d34fc40b3abfdc8610` | architecture_router_declarative | derived_design_obligation | D-0040 | D-0019, D-0037 | none |
| ECM-R-0130 | ECM:L422–L430 | `0d56eda7a5a05c90c3d8d54e2c89b13a31889100e0a0d9d34fc40b3abfdc8610` | architecture_router_declarative | derived_design_obligation | D-0040 | D-0019, D-0037 | none |
| ECM-R-0131 | ECM:L422–L430 | `0d56eda7a5a05c90c3d8d54e2c89b13a31889100e0a0d9d34fc40b3abfdc8610` | architecture_router_declarative | derived_design_obligation | D-0040 | D-0019, D-0037 | none |
| ECM-R-0132 | ECM:L422–L430 | `0d56eda7a5a05c90c3d8d54e2c89b13a31889100e0a0d9d34fc40b3abfdc8610` | architecture_router_declarative | derived_design_obligation | D-0040 | D-0019, D-0037 | none |
| ECM-R-0133 | ECM:L432–L441 | `1461d3e620a0442c10915cf62a3174a8d9d7ee3d656a40dd2a1d38ca2a40d89a` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0008, D-0011, D-0016, D-0037 | none |
| ECM-R-0134 | ECM:L432–L441 | `1461d3e620a0442c10915cf62a3174a8d9d7ee3d656a40dd2a1d38ca2a40d89a` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0037 | none |
| ECM-R-0135 | ECM:L432–L441 | `1461d3e620a0442c10915cf62a3174a8d9d7ee3d656a40dd2a1d38ca2a40d89a` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0026, D-0037 | alias_of ECM-R-0091 |
| ECM-R-0136 | ECM:L432–L441 | `1461d3e620a0442c10915cf62a3174a8d9d7ee3d656a40dd2a1d38ca2a40d89a` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0037 | none |
| ECM-R-0137 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0012, D-0037 | none |
| ECM-R-0138 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0009, D-0012, D-0037 | none |
| ECM-R-0139 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0012, D-0037 | none |
| ECM-R-0140 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0012, D-0037 | none |
| ECM-R-0141 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0012, D-0037 | refines ECM-R-0029 |
| ECM-R-0142 | ECM:L443–L454 | `bc394eba275b1edcccb5c8d4cf1a8b5ea15a74b6bb61d9c865149326ce72e98d` | declarative_durability_or_observability | derived_design_obligation | D-0029 | D-0007, D-0012, D-0018, D-0037 | refines ECM-R-0029 |
| ECM-R-0143 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0037 | none |
| ECM-R-0144 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0022, D-0037 | none |
| ECM-R-0145 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0037 | none |
| ECM-R-0146 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0037 | none |
| ECM-R-0147 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0037 | none |
| ECM-R-0148 | ECM:L456–L469 | `0ca98f20f3c0c1a46dd59f584d59708bedcca11acddf1c9db5f7c9026f26c14b` | declarative_durability_or_observability | derived_design_obligation | D-0030 | D-0037 | none |
| ECM-R-0149 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0037 | none |
| ECM-R-0150 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0037 | none |
| ECM-R-0151 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0037 | none |
| ECM-R-0152 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0037 | none |
| ECM-R-0153 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0016, D-0033, D-0037 | none |
| ECM-R-0154 | ECM:L471–L490 | `ed2fcf6e743ec4198087d69d3cc80e54590eafceecd52e3d9af12c6be199655b` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0019, D-0037, D-0039, D-0040 | none |
| ECM-R-0155 | ECM:L492–L506 | `f9b0f0d69241f5978c9f5af7a1edf5079a05b8fa895cabf4b9c8eec493f92896` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0037, D-0039 | none |
| ECM-R-0156 | ECM:L492–L506 | `f9b0f0d69241f5978c9f5af7a1edf5079a05b8fa895cabf4b9c8eec493f92896` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0019, D-0037, D-0039, D-0040 | none |
| ECM-R-0157 | ECM:L492–L506 | `f9b0f0d69241f5978c9f5af7a1edf5079a05b8fa895cabf4b9c8eec493f92896` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0019, D-0037, D-0039, D-0040 | none |
| ECM-R-0158 | ECM:L492–L506 | `f9b0f0d69241f5978c9f5af7a1edf5079a05b8fa895cabf4b9c8eec493f92896` | required_test_or_breaker | derived_design_obligation | D-0031 | D-0016, D-0037 | none |
| ECM-R-0159 | ECM:L508–L522 | `b578ae77b75028a6c065fe4244a8f69f02b135073a0718e8e9efee2e9b7de61a` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0160 | ECM:L508–L522 | `b578ae77b75028a6c065fe4244a8f69f02b135073a0718e8e9efee2e9b7de61a` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0005, D-0017, D-0018, D-0037 | none |
| ECM-R-0161 | ECM:L508–L522 | `b578ae77b75028a6c065fe4244a8f69f02b135073a0718e8e9efee2e9b7de61a` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0162 | ECM:L508–L522 | `b578ae77b75028a6c065fe4244a8f69f02b135073a0718e8e9efee2e9b7de61a` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0003, D-0015, D-0017, D-0037 | none |
| ECM-R-0163 | ECM:L508–L522 | `b578ae77b75028a6c065fe4244a8f69f02b135073a0718e8e9efee2e9b7de61a` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0023, D-0037 | none |
| ECM-R-0164 | ECM:L526–L537 | `610aaede5b40075079ecd89f22e24b88dc60897ed61f0d867591e261b3349d2b` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0165 | ECM:L526–L537 | `610aaede5b40075079ecd89f22e24b88dc60897ed61f0d867591e261b3349d2b` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0166 | ECM:L526–L537 | `610aaede5b40075079ecd89f22e24b88dc60897ed61f0d867591e261b3349d2b` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0011, D-0017, D-0037 | none |
| ECM-R-0167 | ECM:L539–L541 | `4a517e379f21d05109ab17fc864af9dd9761c97c24ecc03bf6ee2b1ec4d4e633` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0025, D-0037 | none |
| ECM-R-0168 | ECM:L543–L545 | `bbcbc35ff92e38768bfb67b343a52aa1647154efab5d72194534ed5eae6b1414` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0024, D-0037 | none |
| ECM-R-0169 | ECM:L547–L549 | `51736ca59f010ebb5d49b11b95794e4848b625aad971959865dfd64352a282f3` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0170 | ECM:L547–L549 | `51736ca59f010ebb5d49b11b95794e4848b625aad971959865dfd64352a282f3` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0037 | none |
| ECM-R-0171 | ECM:L551–L564 | `0ea8ed2d8121aabf549f3441c8affcc6352921bc072318aec3c486fc376c798f` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0025, D-0037 | none |
| ECM-R-0172 | ECM:L551–L564 | `0ea8ed2d8121aabf549f3441c8affcc6352921bc072318aec3c486fc376c798f` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0003, D-0017, D-0025, D-0037 | none |
| ECM-R-0173 | ECM:L551–L564 | `0ea8ed2d8121aabf549f3441c8affcc6352921bc072318aec3c486fc376c798f` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0017, D-0025, D-0037 | none |
| ECM-R-0174 | ECM:L566–L576 | `11aac2a31a150b5830255b8fb8ab0935b729b2dc7c57baa3c7925c553d08b0f8` | adapter_or_owner_table | derived_design_obligation | D-0032 | D-0009, D-0017, D-0037 | none |
| ECM-R-0175 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0025, D-0037 | none |
| ECM-R-0176 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0012, D-0037 | none |
| ECM-R-0177 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0011, D-0012, D-0029, D-0037 | none |
| ECM-R-0178 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0003, D-0012, D-0037 | none |
| ECM-R-0179 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0003, D-0012, D-0029, D-0037 | none |
| ECM-R-0180 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0026, D-0037, D-0039 | alias_of ECM-R-0135 |
| ECM-R-0181 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0026, D-0037, D-0039 | none |
| ECM-R-0182 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0025, D-0037, D-0039 | alias_of ECM-R-0101 |
| ECM-R-0183 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0032, D-0037, D-0039 | none |
| ECM-R-0184 | ECM:L578–L591 | `bc3a29abc6aa46eac896fa1426c728cc257900a11b747a108ed6dbc328ef1bdd` | adapter_or_owner_table | derived_design_obligation | D-0008 | D-0037, D-0039 | alias_of ECM-R-0037 |
| ECM-R-0185 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0010, D-0037, D-0038, D-0039 | none |
| ECM-R-0186 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0187 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0188 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0189 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0190 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0191 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0192 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0193 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0194 | ECM:L593–L608 | `39f041b418a51b614a5577041a129ed790f32b72338fb995eca099b62e7b5655` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0195 | ECM:L610–L614 | `ab160a34bb4cf37543d8abb1bba8894667c73da1dc3647ca694583e511b34f76` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0010, D-0037, D-0038 | none |
| ECM-R-0196 | ECM:L610–L614 | `ab160a34bb4cf37543d8abb1bba8894667c73da1dc3647ca694583e511b34f76` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0010, D-0037, D-0038 | none |
| ECM-R-0197 | ECM:L610–L614 | `ab160a34bb4cf37543d8abb1bba8894667c73da1dc3647ca694583e511b34f76` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0198 | ECM:L610–L614 | `ab160a34bb4cf37543d8abb1bba8894667c73da1dc3647ca694583e511b34f76` | proposed_implementation_program | derived_design_obligation | D-0034 | D-0037, D-0038 | none |
| ECM-R-0199 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0037 | none |
| ECM-R-0200 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0011, D-0012, D-0029, D-0037, D-0039 | none |
| ECM-R-0201 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0011, D-0037, D-0039 | alias_of ECM-R-0051 |
| ECM-R-0202 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0012, D-0029, D-0037, D-0039 | refines ECM-R-0029 |
| ECM-R-0203 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0003, D-0037, D-0039 | none |
| ECM-R-0204 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0005, D-0018, D-0022, D-0037, D-0039 | none |
| ECM-R-0205 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0007, D-0037, D-0039 | none |
| ECM-R-0206 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0006, D-0037, D-0039 | refines ECM-R-0028 |
| ECM-R-0207 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0037, D-0039 | refines ECM-R-0024 |
| ECM-R-0208 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0016, D-0033, D-0037, D-0039 | none |
| ECM-R-0209 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0003, D-0017, D-0033, D-0037, D-0039 | none |
| ECM-R-0210 | ECM:L616–L631 | `c24e8bf6a4ff3105351eb380ba8b94e9f6f7088816f9a3ad87beb62f2c62b0e8` | acceptance_condition | acceptance_obligation | D-0034 | D-0019, D-0037, D-0039 | none |
| ECM-R-0211 | ECM:L633–L635 | `50db41d59a5da2aa5cf406d3a0a9ad4c821bfc00fca484caf9007d6d209bf8f8` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0212 | ECM:L643–L643 | `943df69ce354d8dd98e3842d16db7066ee5dd34b513f4f14cbd7ed08ba06a1c3` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0213 | ECM:L644–L644 | `40bb05799521b70396eda282f8f76640928863ec6881ce06f11a550dc8913a4f` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0214 | ECM:L645–L645 | `aee5fa450e4f321eec9d10a0b5b52d18c52e1ebf5b6bb845fac8f03cfa30e4bf` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | aggregate_of ECM-R-0129-0131 |
| ECM-R-0215 | ECM:L646–L646 | `82dc7007fd8e6e8202d1fb87fa61813c9a8e015461e29d25c787171b077fe58c` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | alias_of ECM-R-0131 |
| ECM-R-0216 | ECM:L647–L647 | `e624444e999a2be90b656115172cd201428cc2681522a5591af5fc6d4c546387` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0217 | ECM:L648–L648 | `5a842527d96ac896954a16916e52468bde4af017bb43865d232cb80efefaf4c5` | controller_normative | controller_normative | D-0035 | D-0003, D-0037, D-0039 | aggregate_of ECM-R-0021/0047/0063/0100/0162 |
| ECM-R-0218 | ECM:L649–L649 | `b657093e3c702a022afdb2ee3eb3a644cd8386824de550803d77408d334457ab` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | aggregate_of ECM-R-0011/0012/0065/0066 |
| ECM-R-0219 | ECM:L650–L650 | `d8fe42c6b44db7a4f9f4df50cada525a12175402265be3b9f493f73c43a0079b` | controller_normative | controller_normative | D-0035 | D-0006, D-0037, D-0039 | refines ECM-R-0028 |
| ECM-R-0220 | ECM:L651–L651 | `b14e9ade3c39753ef306aa9c004481cfa60650794f62a40722ff206996a74307` | controller_normative | controller_normative | D-0035 | D-0019, D-0037, D-0039 | refines ECM-R-0024 |
| ECM-R-0221 | ECM:L652–L652 | `db1e063edb90fb6ffe3a3d9d0f780d961855bc61414960acfa1ea31ac3dc740b` | controller_normative | controller_normative | D-0035 | D-0007, D-0026, D-0037, D-0039 | aggregate_of ECM-R-0104/0105/0124/0142 |
| ECM-R-0222 | ECM:L653–L653 | `1ec9738b7db49d985103d8f5c634547945b4bec83bdda84a523874f450534dd2` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | aggregate_of ECM-R-0115/0116/0123 |
| ECM-R-0223 | ECM:L654–L654 | `e958ad6a2c52f0a0be06877d994b701be66593072a017eb818450d96a2314d58` | controller_normative | controller_normative | D-0035 | D-0019, D-0037, D-0039 | alias_of ECM-R-0119 |
| ECM-R-0224 | ECM:L655–L655 | `5e7a1544551884307397474f773344a3cb1ee863d5488641919c763d171cb55f` | controller_normative | controller_normative | D-0035 | D-0019, D-0037, D-0039 | aggregate_of ECM-R-0125-0128 |
| ECM-R-0225 | ECM:L656–L656 | `3f165fe6d981fffb8cc5acb31b8c63daf0123e05c01ccccb11bbd46a8fefbe30` | controller_normative | controller_normative | D-0035 | D-0003, D-0037, D-0039 | alias_of ECM-R-0116 |
| ECM-R-0226 | ECM:L657–L657 | `65fa3f3747b8b229c25670d77337742afda15cf76ebe995576dbdc9730e4af31` | controller_normative | controller_normative | D-0035 | D-0011, D-0030, D-0037, D-0039 | aggregate_of ECM-R-0092/0099/0143-0148 |
| ECM-R-0227 | ECM:L658–L658 | `36433ed62f8f367cd4eb68b5283995846ee12c8a39282ab9afbe206737c02cf8` | controller_normative | controller_normative | D-0035 | D-0029, D-0037, D-0039 | aggregate_of ECM-R-0018/0029/0139-0142/0155-0157 |
| ECM-R-0228 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0229 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0021, D-0037, D-0039 | none |
| ECM-R-0230 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | alias_of ECM-R-0083 |
| ECM-R-0231 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0022, D-0037, D-0039 | alias_of ECM-R-0100 |
| ECM-R-0232 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0013, D-0037, D-0039 | aggregate_of ECM-R-0076-0079/0117-0128 |
| ECM-R-0233 | ECM:L660–L666 | `73270107dd9441bad4d87e9e369c7034491ba2e3f3eee9d88e33f1766f94de41` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0234 | ECM:L668–L672 | `67032965180217983b617b97744ee7caa29e3bce54b32e9d7d89ee3b4c15be7c` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | aggregate_of ECM-R-0045/0049 |
| ECM-R-0235 | ECM:L668–L672 | `67032965180217983b617b97744ee7caa29e3bce54b32e9d7d89ee3b4c15be7c` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | alias_of ECM-R-0060 |
| ECM-R-0236 | ECM:L668–L672 | `67032965180217983b617b97744ee7caa29e3bce54b32e9d7d89ee3b4c15be7c` | controller_normative | controller_normative | D-0035 | D-0019, D-0028, D-0037, D-0039 | none |
| ECM-R-0237 | ECM:L668–L672 | `67032965180217983b617b97744ee7caa29e3bce54b32e9d7d89ee3b4c15be7c` | controller_normative | controller_normative | D-0035 | D-0003, D-0028, D-0037, D-0039 | alias_of ECM-R-0107 |
| ECM-R-0238 | ECM:L668–L672 | `67032965180217983b617b97744ee7caa29e3bce54b32e9d7d89ee3b4c15be7c` | controller_normative | controller_normative | D-0035 | D-0028, D-0037, D-0039 | refines ECM-R-0024 |
| ECM-R-0239 | ECM:L674–L684 | `1952643ceaa908643b901d6ab633a7da930716d8caa5b58f3121f5dc47b22b8e` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | broad aggregate_of prior lifecycle obligations |
| ECM-R-0240 | ECM:L674–L684 | `1952643ceaa908643b901d6ab633a7da930716d8caa5b58f3121f5dc47b22b8e` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | alias_of ECM-R-0054 |
| ECM-R-0241 | ECM:L686–L687 | `7d1fa76c153064cc4a4654c7468fca6e6117cee5defa04ba4a14398e80f18cb0` | controller_normative | controller_normative | D-0035 | D-0031, D-0037, D-0039 | aggregate_of ECM-R-0155-0157 |
| ECM-R-0242 | ECM:L686–L687 | `7d1fa76c153064cc4a4654c7468fca6e6117cee5defa04ba4a14398e80f18cb0` | controller_normative | controller_normative | D-0035 | D-0031, D-0037, D-0039 | alias_of ECM-R-0158 |
| ECM-R-0243 | ECM:L689–L690 | `660088819872413593d5b0c647b122ead0f8ada1f06d204d3a543a0527fb2c95` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | none |
| ECM-R-0244 | ECM:L689–L690 | `660088819872413593d5b0c647b122ead0f8ada1f06d204d3a543a0527fb2c95` | controller_normative | controller_normative | D-0035 | D-0037, D-0039 | refines ECM-R-0019/0054/0240 |


## 11. Machine-verification commands and results

Executed against the immutable sources, normalizations, original matrix and this correction:

```text
$ sha256sum <AMX source> <AMX normalization> <ECM source> <ECM normalization> <original Round 2 matrix>
4564e250adbf69832542fb054c43dcef37d944e10fe4d6c482d31ac64ee8c6c9  AMX source
c81aedb9528df2162e5c327f6479a89848e70bf85a3835b3d76b67e5b06dae52  AMX normalization
e2606fd14face691d3d5ef90fbd6727bff69385b0abe6345fb45d132773db980  ECM source
9ddf7754d017384f4d26ef801eac333a8e2a4148ef3d276fd178a032c49c7810  ECM normalization
b3e36bc2fe7e85b6cc485806339e3124c7908734cd4c1c505e514f59c8527837  original Round 2 matrix

$ rg -c 'supersedes=Round2:D-[0-9]{4}' <correction>
40

$ rg -c '^\| AMX-R-[0-9]{4} \|' <correction>
243

$ rg -c '^\| ECM-R-[0-9]{4} \|' <correction>
244

$ comm -23 <published-AMX-ID-set> <corrected-AMX-ledger-ID-set>
# no output

$ comm -23 <published-ECM-ID-set> <corrected-ECM-ledger-ID-set>
# no output

$ git diff --exit-code -- <AMX source path>
# exit 0
```

The expanded-reference verifier parsed the §3 registry and both §10 ledgers, without using numeric ranges for coverage:

```text
registry_rows=40
amx_rows=243
ecm_rows=244
unique_source_spans=131
primary_reference_check=PASS
source_span_digest_recompute=PASS
overall=PASS
```

Every primary record contains its assigned requirement explicitly. All 487 quotation digests were checked by recomputing the 131 unique exact source spans. `AMX-R-0162` is not fabricated or counted.

## 12. Round 4 synthesis safety decision

**Yes—this corrected matrix is safe for Round 4 synthesis as a non-authoritative reconciliation input.** It is not safe for implementation, schema activation, migration, promotion or deployment. Round 4 must preserve the ownership table and carry every §9 critical blocker forward until it is explicitly resolved, rejected or accepted as a documented residual risk.
