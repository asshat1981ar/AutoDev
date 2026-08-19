# Environment profile direction

Represent available capabilities explicitly: OS/architecture, toolchains, filesystem/process/network features, Android/Termux constraints, container/sandbox support, credentials by reference, and remote-companion availability.

Planning/routing uses this profile to avoid impossible tool choices. Environment facts are evidence with freshness; a resumed long-running task may need to refresh them before continuing.
