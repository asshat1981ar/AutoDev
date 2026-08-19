# Policy asset direction

Portable policy assets may express organization/project preferences, deny/allow requests, review requirements, or governance metadata. They must compose beneath a non-removable ForgeCore safety floor.

Imported policy can narrow behavior or request stricter review. It cannot widen kernel capabilities beyond trusted configuration or convert an untrusted permission declaration into `AuthorizationGrant`. Conflicts with mandatory kernel policy fail closed and should be diagnosable to users.
