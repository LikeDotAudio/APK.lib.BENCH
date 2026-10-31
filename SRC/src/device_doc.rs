// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! ONE DEVICE, ONE RETAINED DOCUMENT.
//!
//! An instrument used to be ~30 retained topics under
//! `APK.audio/System/Protocols/visa/Device/<Type>/<Model>/<Dev>/<field>` —
//! `manufacturer`, `model`, `serial`, `raw_idn`, `resource`, `reachable`, … —
//! written by two scanners and two heartbeats, each touching the fields it
//! knew about. Wiping a device meant publishing an empty payload to every leaf
//! anybody had ever written, and every leaf somebody forgot (`status_bit`,
//! `error`, `last_seen`, `presence_topic`) survived as a fossil: the tree read
//! `presence_topic = …gpib7-4…` under a device whose every other field was
//! gone.
//!
//! Now the device node itself carries ONE JSON object, retained:
//!
//! ```text
//! …/Device/DMM/34401A/Dev0   {"manufacturer":"HEWLETT-PACKARD","model":"34401A",
//!                              "resource":"TCPIP::44.44.44.222::gpib7,5::INSTR",
//!                              "connected":1,"reachable":1,"last_online":"-1",…}
//! …/Device/DMM/34401A/Dev0/Write   command channel (unchanged)
//! …/Device/DMM/34401A/Dev0/Read    reply channel (unchanged)
//! ```
//!
//! An empty payload on the node is the device gone. A key that has nothing to
//! say is ABSENT — the object form of "publish empty to delete the leaf".
//!
//! A heartbeat that learns one thing (`reachable` went to 0) cannot publish
//! one field any more; it has to publish the whole document with that field
//! changed. So the process keeps the last document it published for every
//! device here, and [`patch`] is read-modify-publish. Each process owns the
//! documents it wrote: the hunter's scan and heartbeat share one store; the
//! Python plugin's share another.

use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Every per-field leaf the old layout ever published under a device prefix,
/// kept ONLY so a migrating publisher can wipe the fossils once. Nothing
/// publishes these any more.
pub const LEGACY_LEAVES: [&str; 26] = [
    "manufacturer", "model", "serial", "firmware", "raw_idn", "resource",
    "status", "device_type", "notes", "last_online", "online_since",
    "connected", "reachable", "port", "adapter", "usb_serial", "line_state",
    "line_settings", "evidence", "status_bit", "error", "last_seen", "record",
    "presence_topic", "Write", "Read",
];

/// The documents one owner has published, by device topic prefix.
///
/// AN OWNED VALUE (2026-10-30 audit, finding 3). The free functions below
/// ([`put`], [`patch`], [`forget`], [`get`], [`distrust`], [`adopt`]) work on
/// ONE process-global store, [`DocStore::global`], which is how two components
/// in one process touching the same prefix used to share state without
/// knowing it. A component that wants its own documents creates a `DocStore`
/// and passes it; the free functions stay for existing callers and are exactly
/// `DocStore::global().<same method>`.
#[derive(Debug, Default)]
pub struct DocStore {
    docs: Mutex<HashMap<String, Map<String, Value>>>,
}

impl DocStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// The process-wide store the free functions use.
    pub fn global() -> &'static DocStore {
        static STORE: OnceLock<DocStore> = OnceLock::new();
        STORE.get_or_init(DocStore::new)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Map<String, Value>>> {
        self.docs.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Replace this device's document and return the payload to publish.
    pub fn put(&self, prefix: &str, doc: Map<String, Value>) -> String {
        let doc = pruned(doc);
        let payload = Value::Object(doc.clone()).to_string();
        self.lock().insert(prefix.to_string(), doc);
        payload
    }

    /// Merge fields into a device this store holds, and return the whole new
    /// document to publish. `None` for a device it never held — there is no
    /// document to amend, and inventing one from a single field is how a row
    /// with nothing but `reachable = 0` appears — and `None` when the fields
    /// change nothing, so a check-in that learned nothing new says nothing.
    pub fn patch(&self, prefix: &str, fields: &[(&str, Value)]) -> Option<String> {
        let mut docs = self.lock();
        let doc = docs.get_mut(prefix)?;
        let before = doc.clone();
        for (k, v) in fields {
            match v {
                Value::Null => {
                    doc.remove(*k);
                }
                Value::String(s) if s.is_empty() => {
                    doc.remove(*k);
                }
                _ => {
                    doc.insert((*k).to_string(), v.clone());
                }
            }
        }
        if *doc == before {
            return None;
        }
        Some(Value::Object(doc.clone()).to_string())
    }

    /// Forget a device (its node is being wiped).
    pub fn forget(&self, prefix: &str) {
        self.lock().remove(prefix);
    }

    /// The document last published for this device, if any.
    pub fn get(&self, prefix: &str) -> Option<Map<String, Value>> {
        self.lock().get(prefix).cloned()
    }

    /// Withdraw all trust from a device this store holds (see
    /// [`distrust_fields`]). The payload to publish, or `None` when there is
    /// no document or it already said all of that.
    pub fn distrust(&self, prefix: &str) -> Option<String> {
        let doc = self.get(prefix)?;
        self.patch(prefix, &distrust_fields(&doc))
    }

    /// Take a document found on the retained tree (another run wrote it) into
    /// this store WITHOUT its trust (see the free [`adopt`]).
    pub fn adopt(&self, prefix: &str, doc: Map<String, Value>) -> Option<String> {
        self.lock().insert(prefix.to_string(), doc);
        self.distrust(prefix)
    }
}

/// Drop empty strings: an absent key is how the object says "nothing".
fn pruned(mut doc: Map<String, Value>) -> Map<String, Value> {
    doc.retain(|_, v| !matches!(v, Value::String(s) if s.is_empty()) && !v.is_null());
    doc
}

/// [`DocStore::put`] on the process-global store.
pub fn put(prefix: &str, doc: Map<String, Value>) -> String {
    DocStore::global().put(prefix, doc)
}

/// [`DocStore::patch`] on the process-global store.
pub fn patch(prefix: &str, fields: &[(&str, Value)]) -> Option<String> {
    DocStore::global().patch(prefix, fields)
}

/// [`DocStore::forget`] on the process-global store.
pub fn forget(prefix: &str) {
    DocStore::global().forget(prefix)
}

/// [`DocStore::get`] on the process-global store.
pub fn get(prefix: &str) -> Option<Map<String, Value>> {
    DocStore::global().get(prefix)
}

/// A document from `(key, text)` pairs — the shape the scanners hold.
pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, String)>) -> Map<String, Value> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), Value::String(v))).collect()
}

/// Read a device node's payload: the document, or `None` for a wipe or for
/// anything that is not a JSON object.
pub fn parse(payload: &[u8]) -> Option<Map<String, Value>> {
    match serde_json::from_slice::<Value>(payload).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

/// A field as text, whatever JSON type it was written as (`1`, `"1"`, `-1`).
pub fn text(doc: &Map<String, Value>, key: &str) -> String {
    match doc.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

// ─── Trust ──────────────────────────────────────────────────────────────────
//
// NOTHING IS TRUSTED FOR LONGER THAN THIS. A remembered resource, a retained
// document, a roster row: each is only a hint of where to look. An instrument is
// present at a resource only if `*IDN?` answered there, as ITSELF, within the
// last `VERIFIED_TTL_SECS`. The DHCP reshuffle of 2026-09 is why: the 33210A
// moved .155 -> .152, a Chromecast took .155, and every layer that remembered
// ".155 is the generator" went on saying so.
//
// Every document a hunter publishes as present carries `verified_at` (unix
// seconds of the last answer that proved the identity), so any reader can apply
// the same rule without asking the hunter.

/// How long one verification is believed. One named number, read by the scan,
/// the heartbeat, the Discovered table and the presence tracker alike.
pub const VERIFIED_TTL_SECS: u64 = 300;

/// The key carrying the unix time of the last identity-proving answer.
pub const VERIFIED_AT: &str = "verified_at";

/// What an instrument answers when it has no serial to give. `0` from a 34401A
/// behind a GPIB gateway is the query failing politely, not a serial.
///
/// THIS IS THE CONTRACT'S RULE, restated because this crate is serde_json-only:
/// `apkaudio_contracts::identity::is_placeholder_serial` (contracts/rust/src/
/// identity.rs) — all zeros of any length (the empty string included), `none`,
/// `n/a`, `na`, `unknown`, case-insensitive. The test `placeholders_are_the_
/// contract_rule` holds it to `contracts/vectors/identity.json`; widen the
/// contract, never this copy.
const PLACEHOLDER_WORDS: [&str; 4] = ["none", "n/a", "na", "unknown"];

/// Is this serial a placeholder (the contract's `is_placeholder_serial`)?
///
/// Vectors: `SRC/vectors/usable_serial.json` — the ONE file every copy of
/// this rule (this crate; NODE.BareMetal's Python `usable_serial`) tests
/// against.
pub fn is_placeholder_serial(raw: &str) -> bool {
    let t = raw.trim();
    t.chars().all(|c| c == '0') || PLACEHOLDER_WORDS.iter().any(|p| t.eq_ignore_ascii_case(p))
}

/// The serial if it is an identity, `None` for a placeholder.
pub fn usable_serial(raw: &str) -> Option<String> {
    if is_placeholder_serial(raw) { None } else { Some(raw.trim().to_string()) }
}

/// Unix seconds now.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Did the answer just received come from THE instrument this document names?
///
/// A usable serial must match exactly (case-insensitive). With no usable serial
/// on either side, make AND model must match — the best a serial-less box can
/// offer. An answer with no identity at all (an error, an empty `*IDN?`) is
/// never the instrument: "something answered at that address" is precisely the
/// evidence the DHCP reshuffle proved worthless.
pub fn same_instrument(doc: &Map<String, Value>, answer: &Value) -> bool {
    if answer.get("error").is_some() {
        return false;
    }
    let got = |k: &str| answer.get(k).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let (a_model, a_make, a_serial) = (got("model"), got("manufacturer"), got("serial"));
    if a_model.is_empty() || got("raw_idn").is_empty() && a_make.is_empty() {
        return false;
    }
    let d_model = text(doc, "model");
    if !d_model.trim().eq_ignore_ascii_case(&a_model) {
        return false;
    }
    match (usable_serial(&text(doc, "serial")), usable_serial(&a_serial)) {
        (Some(want), Some(have)) => want.eq_ignore_ascii_case(&have),
        // The document knows a serial and the answer gives none: not provably
        // the same box.
        (Some(_), None) => false,
        // The document never had a serial: make + model is all there is.
        (None, _) => {
            let d_make = text(doc, "manufacturer");
            d_make.trim().is_empty() || d_make.trim().eq_ignore_ascii_case(&a_make)
        }
    }
}

/// Is this document's claim of presence still inside the trust window?
///
/// No `verified_at` at all is NOT fresh: a document from before the stamp
/// existed, or one adopted off the retained tree at boot, proves nothing about
/// now.
pub fn is_fresh(doc: &Map<String, Value>, now: u64) -> bool {
    text(doc, VERIFIED_AT)
        .trim()
        .parse::<u64>()
        .is_ok_and(|at| at <= now && now - at <= VERIFIED_TTL_SECS)
}

/// Does this document still say "present" when it may no longer?
pub fn claims_presence(doc: &Map<String, Value>) -> bool {
    matches!(text(doc, "connected").trim(), "1" | "true")
        || matches!(text(doc, "reachable").trim(), "1" | "true")
}

/// The fields that demote a document whose verification has lapsed, or `None`
/// when it is fresh or already says it is not present.
pub fn stale_fields(doc: &Map<String, Value>, now: u64) -> Option<Vec<(&'static str, Value)>> {
    if is_fresh(doc, now) || !claims_presence(doc) {
        return None;
    }
    let mut fields = vec![
        ("connected", Value::from(0)),
        ("reachable", Value::from(0)),
        ("status", Value::from("stale")),
        ("online_since", Value::Null),
    ];
    // `last_online` becomes a time — the last verification, when there was one.
    match text(doc, VERIFIED_AT).trim().parse::<u64>() {
        Ok(at) => fields.push(("last_online", Value::from(at.to_string()))),
        Err(_) => fields.push(("last_online", Value::Null)),
    }
    Some(fields)
}

/// The fields that withdraw ALL trust from a document, fresh or not.
///
/// A SWEEP IS A CLEAN SLATE. When a rescan is asked for — and at boot, before
/// the first one — every instrument found before loses its presence at once
/// and gets it back only by answering `*IDN?` as itself during that sweep.
/// `verified_at` is deliberately left alone, so a reader can still see how old
/// the last proof is; `last_online` becomes that time.
pub fn distrust_fields(doc: &Map<String, Value>) -> Vec<(&'static str, Value)> {
    let mut fields = vec![
        ("connected", Value::from(0)),
        ("reachable", Value::from(0)),
        ("status", Value::from("stale")),
        ("online_since", Value::Null),
    ];
    match text(doc, VERIFIED_AT).trim().parse::<u64>() {
        Ok(at) => fields.push(("last_online", Value::from(at.to_string()))),
        Err(_) => fields.push(("last_online", Value::Null)),
    }
    fields
}

/// [`DocStore::distrust`] on the process-global store.
pub fn distrust(prefix: &str) -> Option<String> {
    DocStore::global().distrust(prefix)
}

/// Take a document found on the retained tree (another run wrote it) into this
/// process's store WITHOUT its trust: nothing from before this process started
/// is present until it answers again. Returns the payload to publish when that
/// changed the document. [`DocStore::adopt`] on the process-global store.
pub fn adopt(prefix: &str, doc: Map<String, Value>) -> Option<String> {
    DocStore::global().adopt(prefix, doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const USABLE_SERIAL_VECTORS: &str = include_str!("../vectors/usable_serial.json");

    /// Two owned stores are two worlds: what one holds the other does not.
    #[test]
    fn an_owned_store_is_not_the_global_one() {
        let a = DocStore::new();
        let b = DocStore::new();
        a.put("x/Dev0", from_pairs([("model", "34401A".to_string())]));
        assert!(b.get("x/Dev0").is_none());
        assert!(get("x/Dev0").is_none(), "leaked into the global store");
        assert!(a.patch("x/Dev0", &[("reachable", json!(1))]).is_some());
        assert!(b.patch("x/Dev0", &[("reachable", json!(1))]).is_none());
        a.forget("x/Dev0");
        assert!(a.get("x/Dev0").is_none());
    }

    /// The placeholder rule is the contract's: every `deviceId` vector in
    /// contracts/vectors/identity.json that carries a serial says whether that
    /// serial was an identity (the id is `proto:<serial>`) or fell through to
    /// the address. Skipped in a standalone clone, which has no contracts/.
    #[test]
    fn placeholders_are_the_contract_rule() {
        // The checked-in vectors (SRC/vectors/usable_serial.json) — the one
        // file every implementation of this rule tests against, in any
        // language. Always present, so this half runs in a standalone clone.
        let vectors: Value = serde_json::from_str(USABLE_SERIAL_VECTORS).unwrap();
        for p in vectors["placeholders"].as_array().unwrap() {
            let p = p.as_str().unwrap();
            assert!(usable_serial(p).is_none(), "{p:?} is a placeholder in the contract");
        }
        for case in vectors["identities"].as_array().unwrap() {
            let (raw, want) = (case["raw"].as_str().unwrap(), case["usable"].as_str().unwrap());
            assert_eq!(usable_serial(raw).as_deref(), Some(want), "{raw:?}");
        }
        let Some(path) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .map(|d| d.join("contracts/vectors/identity.json"))
            .find(|p| p.is_file())
        else {
            eprintln!("contracts/vectors/identity.json not found (standalone clone): skipped");
            return;
        };
        let vectors: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let sanitize = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-') { c } else { '-' })
                .collect()
        };
        let mut seen = 0;
        for v in vectors["deviceId"].as_array().unwrap() {
            let input = &v["input"];
            let Some(serial) = input["serial"].as_str() else { continue };
            let proto = input["protocol"].as_str().unwrap();
            let id = v["deviceId"].as_str().unwrap();
            let by_serial = id == format!("{proto}:{}", sanitize(serial.trim()));
            assert_eq!(usable_serial(serial).is_some(), by_serial, "vector {v}");
            seen += 1;
        }
        assert!(seen >= 5, "identity.json lost its serial vectors");
    }

    #[test]
    fn empty_fields_are_absent_not_empty() {
        let p = put("t/Dev0", from_pairs([("model", "34401A".into()), ("last_online", String::new())]));
        assert_eq!(p, r#"{"model":"34401A"}"#);
    }

    #[test]
    fn patch_rewrites_the_whole_document() {
        put("t/Dev1", from_pairs([("model", "54641D".into()), ("reachable", "1".into())]));
        let p = patch("t/Dev1", &[("reachable", json!(0)), ("status", json!("unreachable"))]).unwrap();
        let doc = parse(p.as_bytes()).unwrap();
        assert_eq!(text(&doc, "model"), "54641D");
        assert_eq!(text(&doc, "reachable"), "0");
        assert_eq!(text(&doc, "status"), "unreachable");
    }

    #[test]
    fn patch_that_changes_nothing_says_nothing() {
        put("t/Dev3", from_pairs([("status_bit", "0".into())]));
        assert!(patch("t/Dev3", &[("status_bit", json!("0"))]).is_none());
    }

    #[test]
    fn patch_never_invents_a_device() {
        assert!(patch("t/never", &[("reachable", json!(0))]).is_none());
    }

    #[test]
    fn patch_with_empty_removes_the_key() {
        put("t/Dev2", from_pairs([("model", "X".into()), ("last_online", "-1".into())]));
        let doc = parse(patch("t/Dev2", &[("last_online", json!(""))]).unwrap().as_bytes()).unwrap();
        assert!(!doc.contains_key("last_online"));
    }

    #[test]
    fn a_wipe_parses_as_nothing() {
        assert!(parse(b"").is_none());
        assert!(parse(b"1").is_none());
        assert!(parse(br#"{"a":1}"#).is_some());
    }

    fn idn(make: &str, model: &str, serial: &str) -> Value {
        json!({"manufacturer": make, "model": model, "serial": serial,
               "raw_idn": format!("{make},{model},{serial},1.0")})
    }

    #[test]
    fn the_same_serial_is_the_same_instrument() {
        let doc = from_pairs([("manufacturer", "Agilent Technologies".into()),
                              ("model", "33210A".into()), ("serial", "MY48000299".into())]);
        assert!(same_instrument(&doc, &idn("Agilent Technologies", "33210A", "MY48000299")));
        // A different box of the same model at the remembered address.
        assert!(!same_instrument(&doc, &idn("Agilent Technologies", "33210A", "MY48000300")));
        // Something else entirely took the lease (the Chromecast at .155).
        assert!(!same_instrument(&doc, &idn("Google", "Chromecast", "X")));
        // Nothing answered as anything.
        assert!(!same_instrument(&doc, &json!({"error": "timeout"})));
        assert!(!same_instrument(&doc, &json!({})));
        // Model right, serial missing: not provably the same box.
        assert!(!same_instrument(&doc, &idn("Agilent Technologies", "33210A", "0")));
    }

    #[test]
    fn a_serial_less_instrument_is_matched_on_make_and_model() {
        let doc = from_pairs([("manufacturer", "HEWLETT-PACKARD".into()),
                              ("model", "34401A".into()), ("serial", "0".into())]);
        assert!(same_instrument(&doc, &idn("HEWLETT-PACKARD", "34401A", "0")));
        assert!(!same_instrument(&doc, &idn("HEWLETT-PACKARD", "34970A", "0")));
        assert!(!same_instrument(&doc, &idn("KEITHLEY", "34401A", "0")));
    }

    #[test]
    fn verification_lapses_after_the_ttl() {
        let now = 1_000_000;
        let at = |t: u64| from_pairs([("connected", "1".into()), (VERIFIED_AT, t.to_string())]);
        assert!(is_fresh(&at(now), now));
        assert!(is_fresh(&at(now - VERIFIED_TTL_SECS), now));
        assert!(!is_fresh(&at(now - VERIFIED_TTL_SECS - 1), now));
        // No stamp at all proves nothing.
        assert!(!is_fresh(&from_pairs([("connected", "1".into())]), now));
        // Lapsed and still claiming presence -> demoted to stale.
        let fields = stale_fields(&at(now - 301), now).expect("lapsed");
        assert!(fields.iter().any(|(k, v)| *k == "status" && v == "stale"));
        assert!(fields.iter().any(|(k, v)| *k == "connected" && v == 0));
        // Fresh, or already not present -> nothing to say.
        assert!(stale_fields(&at(now), now).is_none());
        assert!(stale_fields(&from_pairs([("connected", "0".into())]), now).is_none());
    }

    #[test]
    fn a_retained_document_is_adopted_without_its_trust() {
        let now = 2_000_000;
        let old = from_pairs([("model", "N9340B".into()), ("connected", "1".into()),
                              ("reachable", "1".into()), ("last_online", "-1".into())]);
        let payload = adopt("t/adopt0", old).expect("an unstamped claim is demoted");
        let doc = parse(payload.as_bytes()).unwrap();
        assert_eq!(text(&doc, "connected"), "0");
        assert_eq!(text(&doc, "status"), "stale");
        assert!(!doc.contains_key("last_online"));
        // Even one verified seconds ago: boot is a clean slate. Its proof's
        // age stays readable.
        let fresh = from_pairs([("model", "X".into()), ("connected", "1".into()),
                                ("reachable", "1".into()), (VERIFIED_AT, (now - 10).to_string())]);
        let doc = parse(adopt("t/adopt1", fresh).unwrap().as_bytes()).unwrap();
        assert_eq!(text(&doc, "connected"), "0");
        assert_eq!(text(&doc, "reachable"), "0");
        assert_eq!(text(&doc, VERIFIED_AT), (now - 10).to_string());
        assert_eq!(text(&doc, "last_online"), (now - 10).to_string());
        // And the store holds it, so the heartbeat can restore it later.
        assert!(patch("t/adopt1", &[("connected", json!(1))]).is_some());
    }

    #[test]
    fn a_sweep_withdraws_trust_from_every_published_document() {
        put("t/sweep0", from_pairs([("model", "33210A".into()), ("connected", "1".into()),
                                    ("reachable", "1".into()), ("status", "identified".into()),
                                    (VERIFIED_AT, "123".into()), ("last_online", "-1".into())]));
        let doc = parse(distrust("t/sweep0").unwrap().as_bytes()).unwrap();
        assert_eq!(text(&doc, "connected"), "0");
        assert_eq!(text(&doc, "reachable"), "0");
        assert_eq!(text(&doc, "status"), "stale");
        assert_eq!(text(&doc, VERIFIED_AT), "123", "the proof's age stays readable");
        assert_eq!(text(&doc, "last_online"), "123");
        // Twice says nothing new; a device never published has nothing to lose.
        assert!(distrust("t/sweep0").is_none());
        assert!(distrust("t/never-published").is_none());
    }
}
