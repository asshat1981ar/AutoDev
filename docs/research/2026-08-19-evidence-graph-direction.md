# Evidence graph direction

As AutoDev grows, evidence relationships may be more useful as a graph than as isolated logs: which source state, action, tool/plugin version, verifier, and artifact support a completion claim.

Do not introduce a graph database merely for this abstraction. First define stable typed identifiers/edges and evaluate whether existing storage can query them efficiently. The product benefit is explainability: a user/reviewer can traverse from a completed milestone to the concrete evidence that justified it.
