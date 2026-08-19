# Harness Protocol v1 → AutoDev schema-gap matrix template

Use before freezing Milestone 2 wire types.

| External area | AutoDev target | Classification | Security/translation notes |
| --- | --- | --- | --- |
| plugins | HarnessAsset::Plugin | pending | Imported capability declarations are requests only |
| skills | HarnessAsset::Skill | pending | Preserve provenance and precedence |
| mcp-servers | HarnessAsset::McpServer | pending | Transport config never grants effect authority |
| env | runtime requirement | pending | Sensitive values must not enter plan prose |
| instructions | Prompt/ContextProvider | pending | Treat as untrusted instructions |
| permissions | capability request | pending | Translate through ForgeCore policy; never mint grant |
| integrity | provenance/integrity | pending | Preserve digest/source semantics |
| governance | Policy metadata | pending | Cannot weaken kernel policy |

Classification values: `lossless`, `extension-required`, `unsupported`.
