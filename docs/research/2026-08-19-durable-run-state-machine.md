# Durable run state-machine sketch

This is a research sketch, not yet an implementation contract.

```text
planned -> running -> completed
             |  |
             |  +-> blocked -> running
             +----> interrupted -> reconcile -> running
             +----> failed
             +----> cancelled
```

Effectful execution has a separate envelope lifecycle. The durable plan state must not collapse these layers. In particular, `interrupted` means coordination knows the run stopped but does not assume whether the last external effect happened. Reconciliation resolves that uncertainty before further effectful work.

Later orchestration work should model reconciliation evidence explicitly rather than relying only on a boolean API parameter.
