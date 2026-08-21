# Mistral Connector Tool Snapshots

This directory stores **sanitized** tool-schema snapshots captured from Mistral after a Connector is registered and authenticated.

No live snapshot is committed in the initial implementation because no Mistral account mutation or credential connection has been approved in this development session.

## Capture rule

For each live Connector:

1. call the documented list-tools endpoint with `refresh=true`;
2. remove credential/header/token material;
3. store only tool name, description, input schema, and non-secret annotations needed for policy review;
4. name the file `<connector-key>.tools.json`;
5. compare it against the prior version with `mistral_connector_sync.py diff-tools`;
6. treat added or changed tools as denied until reviewed;
7. commit the reviewed snapshot together with the exact permission-policy change.

Snapshots are evidence, not authority. A new tool appearing in a snapshot does not make it available to an Agent automatically.