# Repository Context Fabric

AutoDev needs a repository-exploration layer between the task planner and the model fabric. The context fabric is the first implementation of that boundary.

## Why it exists

Repository-scale coding agents are limited by context selection as much as by generation. Feeding an entire repository to a model is expensive and often lowers signal-to-noise. AutoDev therefore treats repository exploration as a separate, observable capability.

The first implementation is intentionally local-first and deterministic:

```text
Task / issue
    |
    v
Context query
    |
    v
Repository snapshot -> lexical ranking -> bounded ContextPack
                                      |
                                      v
                               ModelRequest
```

`ContextPack` contains ranked files, relevance scores, reasons for selection, and a hard byte budget. No model call or embedding service is required.

## Why lexical retrieval first

This establishes a stable baseline before adding embeddings or learned reranking. It is cheap enough for every planning step, deterministic enough for tests, and works offline. It also gives later retrieval systems a measurable baseline to beat.

The intended evolution is:

1. deterministic lexical retrieval;
2. language-aware symbol extraction;
3. optional local embeddings/reranking;
4. model/provider-specific context budgeting;
5. retrieval evaluation using successful task trajectories.

## Model integration

Qwen3-Coder is a useful target for the next model-fabric iteration because its published design emphasizes agentic coding and repository-scale context, including 256K native context and longer-context extension. AutoDev should not hard-code that model, however. The `ContextPack` boundary remains provider-neutral.

## Safety properties

The context fabric is read-only. It does not execute commands, modify files, invoke network services, or make policy decisions. Its output is evidence for the planning/model layers.

The byte and file limits are enforced before a pack is returned. Ranking is deterministic: equal scores are ordered by workspace-relative path.

## Future extension: retrieval evidence

Each selected item already carries `reasons`. These should eventually become structured retrieval evidence with fields such as retrieval method, query tokens, symbol matches, embedding score, and source revision. That will let verification and provenance answer not only "what did the agent change?" but also "what repository evidence informed the change?".
