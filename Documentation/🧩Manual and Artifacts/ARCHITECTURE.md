# LIB.BENCH architecture

## Using it

Add the crate by path from a member of the `DISCOVERY.Engine/SRC` workspace:

```toml
apkaudio-bench = { path = "../../../LIBRARY/LIB.BENCH/SRC" }
```

Current users: `PLUGIN.VISA/SRC/Cargo.toml` and
`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/Cargo.toml`. The `apkaudio_visa` crate
re-exports `device_doc` under its old name.

## Parts

| Module | File | What it holds |
| :-- | :-- | :-- |
| `device_doc` | `SRC/src/device_doc.rs` | The per-device retained JSON document: build, patch, parse, freshness and trust. |
| `dev_id` | `SRC/src/dev_id.rs` | The `Dev<id>` segment of a device topic, derived from the VISA resource and serial. |
| `declared` | `SRC/src/declared.rs` | Instruments that cannot be probed (listen-only, such as the HP 3235 switch matrix), read from their capability sheets. |
| `trigger` | `SRC/src/trigger.rs` | Whether a GUI payload means "go". |

`SRC/src/lib.rs` only declares the four modules.

## Data flow

The topic tree is `APK.audio/System/Protocols/visa/Device/<Type>/<Model>/Dev<id>`.

1. **Scan (writer, PLUGIN.VISA).** The plugin asks `*IDN?` at a resource. It names the device
   with `dev_id(resource, serial)`, checks the answer with `same_instrument`, stamps
   `verified_at`, and publishes the string returned by `put` as one retained payload on the
   device node. `Write` and `Read` below the node remain command and reply channels.
2. **Heartbeat (writer).** When one field changes (`reachable` goes to 0), the writer calls
   `patch(prefix, fields)`: read the last document it published from the process store, merge,
   prune empty keys, return the whole payload to publish. When `stale_fields` reports a lapsed
   verification, the writer patches presence away.
3. **Rescan or boot (writer).** `distrust(prefix)` removes presence from every held document;
   instruments regain it only by answering as themselves. Documents found on the retained tree
   from an earlier run enter the store through `adopt`, without trust.
4. **Declared instruments (writer).** `declared_devices(library)` lists sheets that carry a
   `declared` block (`resource`, `listen_only`, `on_connect`); `on_connect` is resolved to SCPI
   from the model's own vocabulary (`komand_scpi`). The plugin publishes them as present without a
   probe.
5. **Panel builder (reader, orchestrator).** It `parse`s each retained document, reads fields with
   `text`, checks `is_fresh`, and reads every capability sheet through `sheets(library)` to build
   the instrument panel.
6. **Buttons (both).** Rescan and Clear presses arrive as GUI payloads; each process decides with
   `trigger::is_truthy`.

## Tests

`cargo test -p apkaudio-bench` from `PODS/POD.THICK/DISCOVERY.Engine/SRC`. Tests are inline in
each module under `#[cfg(test)]`.
