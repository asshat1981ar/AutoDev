# MCP integration direction

MCP remains a tool/context interoperability layer, not a trust boundary. A future HarnessAsset::McpServer should capture transport/configuration/provenance and requested capabilities while actual effects remain subject to AutoDev policy.

Prefer modern stateless/streamable HTTP paths for remote/mobile-compatible integrations where supported, while retaining stdio adapters for local developer environments. Android-first workflows must not assume a desktop daemon or native PTY.

Imported MCP definitions from Harness Protocol or other ecosystems require validation before activation and must never smuggle trusted approval state through environment/configuration fields.
