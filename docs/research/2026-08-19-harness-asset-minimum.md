# Minimum HarnessAsset metadata hypothesis

Every normalized harness asset likely needs:

- stable `id` and human name;
- asset `kind`;
- semantic/schema version;
- source/provenance;
- integrity metadata when externally materialized;
- compatibility constraints;
- configuration schema/defaults;
- requested capabilities/side-effect class where relevant;
- trust classification as metadata, not authority;
- dependencies on other assets;
- optional external-format identity for round-trip import/export.

This remains a hypothesis until the Harness Protocol schema-gap analysis is complete.
