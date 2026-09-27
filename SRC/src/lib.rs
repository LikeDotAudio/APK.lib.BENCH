// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The bench's shared vocabulary — what the VISA plugin writes and the
//! orchestrator's panel builder reads (PLAN-3316.01 step 6.6).
//!
//! Three things, each with one writer and one or more readers in DIFFERENT
//! processes, which is why none of them may live in either process:
//!
//! - [`device_doc`] — one instrument, one retained JSON document. Was
//!   `apkaudio_visa::device_doc`; that crate re-exports it under the old name.
//! - [`declared`] — instruments wired to the bench that no probe can find,
//!   read from their capability sheets in the instrument library.
//! - [`trigger`] — whether a GUI press payload means "go".
//! - [`dev_id`] — the `Dev<id>` in a device topic, read off the instrument's
//!   address (or LAN serial) so every process spells it the same way.

pub mod declared;
pub mod dev_id;
pub mod device_doc;
pub mod trigger;
