# Configuration drift direction

A durable run must detect when material configuration changes between checkpoint and resume: repository base, policy version, plugin/tool integrity, workflow/profile version, verifier recipe, or environment capabilities.

Not every change requires aborting. The recovery layer classifies drift as compatible, requires revalidation/replan, or unsafe to resume. The classification and decision become durable evidence rather than silently continuing under changed assumptions.
