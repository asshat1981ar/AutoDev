# Telemetry direction

Telemetry should answer operational/evaluation questions with minimal sensitive content: lifecycle timings, failure classes, verifier outcomes, resource/context usage, recovery/intervention counts, configuration identities/hashes, and platform capability state.

Source code/prompts/transcripts should not be collected by default merely because they are easy to log. Local-first/privacy policy governs whether richer traces may leave the device/project environment.
