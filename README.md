# LIB.BENCH — the instrument bench's shared vocabulary

| | |
|---|---|
| **Crate** | `apkaudio-bench` (`SRC/`), member of the `discovery/SRC` workspace |
| **Container** | none — a library, like the other `lib:*` stacks |
| **Written by** | `PLUGIN.VISA` (`apk-visa`) |
| **Read by** | the orchestrator's panel builder (`BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA`) |

What the VISA plugin writes and the panel builder reads, in one place because since
PLAN-3316.01 (step 6.6) they are two processes:

- `device_doc` — one instrument, one retained JSON document (was `apkaudio_visa::device_doc`,
  still re-exported there under that name).
- `declared` — instruments wired to the bench that no probe can find, read from their
  capability sheets in `PODS/POD.THICK/INSTRUMENTS`.
- `trigger` — whether a GUI press payload (`{"value":1}`, `1`, `true`, `scan`) means "go".

Only `serde_json`: a reader of the bench must not inherit what it takes to drive one.
There is deliberately no `config.ini` here — this is not a module and publishes nothing.
