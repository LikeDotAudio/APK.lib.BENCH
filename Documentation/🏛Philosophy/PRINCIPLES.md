# LIB.BENCH principles

LIB.BENCH exists because the instrument bench is written by one process (the VISA plugin) and read
by another (the orchestrator's panel builder). Since PLAN-3316.01 step 6.6 they no longer share an
address space, so whatever both must agree on — the shape of a device document, the declared
instruments, what counts as a button press — lives here, once, and in neither process.

## The rules it keeps

1. **A reader of the bench does not inherit what it takes to drive one.** The only dependency is
   `serde_json` (`SRC/Cargo.toml:16`). No socket, no async runtime, no VISA driver.
2. **One device, one retained document.** A device is one JSON object on its node, not ~30 leaf
   topics. `put` (`SRC/src/device_doc.rs:62`) and `patch` (`SRC/src/device_doc.rs:74`) always
   publish the whole document. The old leaf names survive only in `LEGACY_LEAVES`
   (`SRC/src/device_doc.rs:42`) so a migrating publisher can wipe them once.
3. **An absent key means nothing to say.** Empty strings and nulls are pruned before publishing
   (`pruned`, `SRC/src/device_doc.rs:56`).
4. **Unreachable is dead; trust decays.** A verification is believed for `VERIFIED_TTL_SECS` = 300 s
   (`SRC/src/device_doc.rs:145`). A document with no `verified_at` is never fresh (`is_fresh`,
   `SRC/src/device_doc.rs:212`). A document found on the retained tree at boot is adopted without
   its trust (`adopt`, `SRC/src/device_doc.rs:278`). A rescan withdraws all presence at once
   (`distrust_fields`, `SRC/src/device_doc.rs:252`).
5. **Identity is the serial, never the address.** `same_instrument` (`SRC/src/device_doc.rs:181`)
   requires a matching usable serial, or matching make and model when neither side has one; an
   answer with no identity is never the instrument. A placeholder serial such as `0` is not an
   identity (`usable_serial`, `SRC/src/device_doc.rs:157`).
6. **A device id names where the instrument is, not the order it was heard in.** `dev_id`
   (`SRC/src/dev_id.rs:27`) reads it off the VISA resource: a GPIB or serial address is a cable and
   names the device; a LAN address is a lease, so a LAN instrument is named for its serial.
7. **The library directory is the argument, not the repository.** Every function in `declared`
   takes the instrument library path (`sheets`, `SRC/src/declared.rs:62`;
   `declared_devices`, `SRC/src/declared.rs:150`). Finding it is the caller's business. The first
   sheet for a model key wins, in sorted order, so two benches on one commit agree.
8. **One rule for "go".** `is_truthy` (`SRC/src/trigger.rs:12`) decides whether a GUI payload
   (`{"value":1}`, `1`, `true`, `scan`) is a press, for every process that listens for one.
9. **No `config.ini`.** This is a library, not a module; it has no tunables and publishes nothing.

## Where it breaks them

- `SRC/Cargo.toml` names `workspace = "../../../DISCOVERY.Engine/SRC"`, so the crate builds only
  inside an APK.audio checkout, not from this repository alone.
- The test `the_shipped_library_declares_the_switch_matrix` (`SRC/src/declared.rs:213`) reads the
  real instrument library at `../../../INSTRUMENTS` and silently passes when it is absent.
- The document store is a process-wide `Mutex<HashMap>` (`SRC/src/device_doc.rs:50`); each process
  owns the documents it wrote, and two writers of one device in one process share that store.
