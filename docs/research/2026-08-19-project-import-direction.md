# Project import direction

Project import should inspect repository structure/instructions/build systems and create a bounded environment profile without modifying the project by default. It can propose detected verification commands, platform capabilities, and harness assets for review.

Imported repository instructions/context are untrusted project inputs and cannot weaken AutoDev's kernel policy. Diagnostics should flag unsupported build/runtime requirements early, especially on Android/Termux.
