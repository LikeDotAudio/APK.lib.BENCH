# 🛣 Improvements — LIB.BENCH (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

Delivered items are recorded in [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Tag releases (`bench-v0.x`) and have PLUGIN.VISA and the orchestrator depend on the same tag instead of a `rev` and a relative path. **Needs owner:** push this repository and publish tag `bench-v0.2.0` (the vectors, fixture and `DocStore` landed 2026-10-30; bump `SRC/Cargo.toml` with the engine and Core lockfiles in the same change); then the consumer repins (handoffs filed) | Change management | S | `SRC/Cargo.toml`; PLUGIN.VISA `rev = 9b71355…`; `ORCHESTRATOR:hunter:VISA/Cargo.toml` path dep |
