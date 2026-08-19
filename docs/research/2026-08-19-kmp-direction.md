# KMP direction

KMP should carry portable control-plane domain/view contracts useful to Android and other clients: run summaries, task/evidence/approval models, event DTOs, and API abstractions. Platform-specific filesystem/process/security primitives remain behind platform implementations.

Do not move ForgeCore trusted execution semantics into commonMain merely for code sharing. KMP is a portable client/control-plane layer, not the Rust kernel replacement.
