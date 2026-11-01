# LIB.BENCH delivery

LIB.BENCH is not shipped on its own. It is compiled into the binaries that depend on it.

- **Repository:** https://github.com/LikeDotAudio/APK.lib.BENCH (private), a submodule of the
  APK.audio monorepo at `PODS/POD.THICK/LIBRARY/LIB.BENCH`.
- **Build:** standalone (`cargo test` in `SRC/` — the one check; `serde_json` is the only
  dependency), and as a member of the Cargo workspace at `PODS/POD.THICK/DISCOVERY.Engine/SRC`
  through the link `SRC/members/LIB.BENCH`. PLUGIN.VISA takes it by git `rev` (patched back to the
  local path in the engine); the orchestrator by relative path.
- **Images:** the build stages that compile the workspace copy `PODS/POD.THICK/LIBRARY/LIB.BENCH/SRC`
  into the image: `PODS/POD.THICK/NODE.BareMetal/Docker/Dockerfile` and
  `PODS/POD.databus/DATABUS.Broker.MQTT/Docker/Dockerfile`. Only `SRC/` is copied; the root
  `.dockerignore` excludes every `.git`.
- **Where it lands:** inside the VISA plugin (`apk-visa`) and the orchestrator's VISA hunter
  binaries. No container, port, volume or config file of its own.
- **After an edit:** rebuild the images above (`up-stack.sh` for the stacks that carry them), then
  commit in this repository and bump the submodule pointer in APK.audio.

## Delivered 2026-10-31: portable file names (no `:` `|` or case twins)

- The monorepo now checks out on Windows and macOS: no tracked name holds `<>:"|?*`, a trailing dot or space, a device name or a case-only twin. Colon folders became dot folders (the POD.x convention).
- References to the renamed folders updated (`Baremetal.Manager`, `ORCHESTRATOR.hunter.VISA`). Docs/comments only, no behaviour change.

## Delivered from the 2026-10-30 audit

- **The serial rule as data** — `SRC/vectors/usable_serial.json` (version 1): the rule in words, 18 placeholder spellings and 9 identities with the expected `usable_serial` output (trimmed, case preserved), including the two spellings (`null`, `-`) NODE.BareMetal's Python treats differently, marked as drift. `device_doc` test `placeholders_are_the_contract_rule` now reads it with `include_str!`, so it always runs (it still also checks `contracts/vectors/identity.json` when the monorepo is around). NODE.BareMetal's `instrument_launcher.py` `usable_serial` testing against the same file (or calling `apkaudio_contracts.identity.is_placeholder_serial`): handoff filed.
- **Checked-in sheet fixture** — `SRC/fixtures/library/` replaces the `CARGO_MANIFEST_DIR/../../../INSTRUMENTS` lookup that skipped silently in a standalone clone; `the_fixture_library_declares_the_switch_matrix` checks the declared block, `listen_only`, `on_connect` resolved through the KOMAND table (and an unknown command skipped), the manufacturer-prefixed folder, and sheet order.
- **Owned document store** — `device_doc::DocStore` (`new`, `global`, `put`, `patch`, `forget`, `get`, `distrust`, `adopt`); the free functions are now exactly `DocStore::global().…`, so PLUGIN.VISA and the orchestrator compile unchanged. Test `an_owned_store_is_not_the_global_one`. Callers moving to owned stores where two components share a process: handoff to PLUGIN.VISA filed.
- **Tests run per change as one command** — no workflow here (the root CI verifies nothing on purpose); the check is `cargo test` in `SRC/`. Verified: a copy of `SRC/` alone builds and passes 21 tests offline; `cargo test -p apkaudio-bench` in the engine workspace passes too.
- **Released `0.2.0`, tag `bench-v0.2.0`** — `SRC/Cargo.toml` bumped 0.1.0 → 0.2.0 (vectors, fixture, `DocStore`), annotated tag `bench-v0.2.0` published (owner-authorised 2026-10-31). PLUGIN.VISA repins `rev = "9b71355…"` → `tag = "bench-v0.2.0"` (handoff); the orchestrator keeps its path dependency inside the monorepo; engine/Core lock refresh handed to the coordinator. Verified: `git archive HEAD` of `SRC/` builds and tests alone.

## Releases
- **Released `2.0.0`, tag `v2.0.0`** (2026-11-01, ROAD-3452, the owner's V2) — no SDK pin; every manifest says 2.0.0; GitHub release `v2.0.0`.
