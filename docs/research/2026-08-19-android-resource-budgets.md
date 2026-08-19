# Android resource-budget direction

Later Android/runtime work should make resource constraints explicit: battery/background limits, network availability, storage, thermal/CPU cost, and model/runtime footprint. Heavy compilation/model execution may be delegated while the phone retains durable control/review capability.

Background work should use platform-appropriate scheduling rather than indefinite foreground assumptions. Resource policy should be observable so a run can explain why it deferred or moved work to a companion.
