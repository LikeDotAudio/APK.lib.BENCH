# 🔍 Architecture audit — LIB.BENCH (2026-10-30)

Audited against the APK.audio multi-repo architecture standard: coupling, boundaries, duplication, contracts.
← [Audits](INDEX.md) · Improvements: [🛣Roadmap/IMPROVEMENTS.md](../%F0%9F%9B%A3Roadmap/IMPROVEMENTS.md)

## Bounded context
The instrument bench's shared vocabulary: the one-retained-JSON-per-device document, `Dev<id>` naming, declared-instrument sheets and the GUI trigger rule. Rust library crate `apkaudio-bench`, ~840 lines in 4 modules, one dependency (`serde_json`). No process, no socket. Sampled: all of `SRC/`.

## Context map
| Talks to | How (topic / endpoint / file / import) | Relationship |
|---|---|---|
| PLUGIN.VISA (`apk-visa`) | Rust dependency, writes the device document | Shared Kernel (supplier) |
| NODE.BareMetal `ORCHESTRATOR:hunter:VISA` | Rust dependency, `path = "../../../../LIBRARY/LIB.BENCH/SRC"` in its Cargo.toml | Shared Kernel (supplier) |
| DISCOVERY.Engine workspace | member via symlink `members/LIB.BENCH`, `[patch]` of the git URL | Build host |
| INSTRUMENTS library | reads `INSTRUMENT.<Family>/<Model>/<Model>.gui` and `.komand` from a directory the caller passes (`SRC/src/declared.rs:62`, `:131`) | Conformist to the instrument sheet format |
| Bus topics `…/Protocols/visa/Device/<Type>/<Model>/<Dev>` | the payload shape this crate defines (`SRC/src/device_doc.rs:15-24`) | Published Language |
| NODE.BareMetal Python (`Baremetal:Manager/instrument_launcher.py`) | re-implements part of the same rules in Python | Duplicate (see finding 4) |

## Scorecard
| Category | Score (0–3) | Finding |
|---|---|---|
| Coupling & Cohesion | 3 | Library takes the instrument library path as a parameter; tests use a checked-in fixture; the document store is an owned `DocStore` (the global one remains for existing callers). |
| Boundary Integrity | 3 | `serde_json` only; it *is* the translation layer between the bench and the bus document. |
| Duplication Control | 2 | The serial rule is published as a vector file; NODE.BareMetal's Python copy still differs until it adopts it (handoff filed). |
| Testing & Contracts | 3 | 21 unit tests incl. payload shape, a fixture library and the serial vectors; one standalone command (`cargo test` in `SRC/`). |
| Change management | 2 | `LEGACY_LEAVES` documents the old per-field layout for a one-time wipe (good deprecation). Crate stays at 0.1.0, untagged; consumers pin by `rev` or path. |

Score key: 3 meets the standard · 2 minor gaps · 1 significant gaps · 0 absent / violated.

## Findings
1. **Good — the shared kernel is deliberately small.** `SRC/Cargo.toml` allows `serde_json` and nothing else so a reader of the bench never links a driver; `SRC/src/lib.rs:1-20` names one writer and the readers in other processes for each module.
2. **Good — deprecation of the old topic layout is explicit.** `SRC/src/device_doc.rs` keeps `LEGACY_LEAVES` only so a migrating publisher can wipe the ~26 fossil leaves of the old per-field layout once.
3. **Rule re-implemented outside the crate (LIB side delivered).** `usable_serial` is re-implemented in `PODS/POD.THICK/NODE.BareMetal/SRC/Baremetal:Manager/instrument_launcher.py:515` with a wider list (`null`, `-`) and upper-casing. `SRC/vectors/usable_serial.json` is now the shared data both test against; the Python side is handed off.
4. **No versioned release (needs owner).** `SRC/Cargo.toml` is `0.1.0` and the repo has no tags; PLUGIN.VISA pins `rev = "9b71355…"` while the orchestrator uses a relative path.
