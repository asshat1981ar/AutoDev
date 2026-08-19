# Context freshness direction

Context/evidence that can change externally should carry freshness or source-state identity. A long-running plan may need to re-fetch repository status, issues/PRs, dependency docs, environment capability, or remote tool state before acting on old context.

Freshness checks should be proportional to risk/cost; immutable source blobs/commits need less revalidation than mutable remote state.
