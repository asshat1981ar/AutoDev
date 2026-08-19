# Tool asset direction

A Tool asset describes a callable capability surface: stable identity/version, schema, provenance, compatibility, transport/runtime requirements, side-effect classification, and requested capabilities.

Tool discovery and description are separate from authorization. Runtime invocation still becomes typed intent evaluated by ForgeCore. Tool descriptions and schemas are untrusted metadata and should not be able to override policy classifications silently.
