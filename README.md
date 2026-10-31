# LIB.BENCH — the instrument bench's shared vocabulary

`apkaudio-bench` (`SRC/`) is the one place where the VISA plugin, which writes the instrument bench,
and the orchestrator's panel builder, which reads it, agree on what an instrument looks like. It is
a Rust library with one dependency, `serde_json`. It runs no process, opens no socket and publishes
nothing.

| | |
|---|---|
| **Crate** | `apkaudio-bench`, a member of the `PODS/POD.THICK/DISCOVERY.Engine/SRC` workspace |
| **Container** | none — a library, like the other `LIB.*` repos |
| **Written by** | `PLUGIN.VISA` (`apk-visa`) |
| **Read by** | `NODE.BareMetal/SRC/Core/ORCHESTRATOR.hunter.VISA` |

- [Principles](Documentation/🏛Philosophy/PRINCIPLES.md) — the rules this crate keeps, and where it breaks them.
- [Architecture](Documentation/🧩Manual and Artifacts/ARCHITECTURE.md) — the four modules and how a device document flows through them.
- [Delivery](Documentation/🌍Publishing/DELIVERY.md) — how it is built and which images carry it.

Remote: https://github.com/LikeDotAudio/APK.lib.BENCH

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 13:11 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
