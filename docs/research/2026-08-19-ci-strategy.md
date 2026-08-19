# CI verification strategy for this slice

The current execution container cannot clone the repository over the network, so it cannot produce trustworthy local cargo/Python evidence for GitHub-only edits. The feature branch therefore carries a focused GitHub Actions workflow that runs the exact new Rust and Python gates in a repository checkout.

This is not treated as success until workflow results are fetched and reviewed. Existing repository CI on the pull request remains an additional regression gate.
