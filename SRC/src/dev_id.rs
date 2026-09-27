// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The last segment of `…/Device/<type>/<model>/Dev<id>` — NAMED FOR WHERE THE
//! INSTRUMENT IS, never for the order it was heard in.
//!
//! It used to be a per-model counter over one pass's discovery order, so two
//! 54641Ds on one gateway swapped `Dev0` and `Dev1` from pass to pass and a
//! command for one ran on the other. The id is now read straight off the
//! instrument, the same way by every process that spells a device topic:
//!
//!   `TCPIP::gw::gpib7,16::INSTR`     → `Dev716`    board 7, primary 16
//!   `TCPIP::gw::gpib7,30,1::INSTR`   → `Dev730-1`  …secondary 1 — the `-`
//!                                      keeps 1,11 and 11,1 apart
//!   `ASRL/dev/ttyUSB0::INSTR`        → `DevttyUSB0`
//!   `USB0::0x0957::0xFFEF::CN03480580::INSTR` → `DevCN03480580`
//!   `TCPIP::44.44.44.157::INSTR`     → `Dev<serial>` when it gave a real one,
//!                                      `Dev44-44-44-157` when it did not
//!
//! A LAN instrument is named for its SERIAL because its address is a lease: the
//! same box on a new IP is the same instrument. A GPIB or serial instrument is
//! named for its address because that address is a cable — the user's own
//! wiring — and two identical meters on it are told apart by nothing else.

use crate::device_doc::usable_serial;

/// `Dev<id>` for the instrument at `resource` that answered `serial`.
pub fn dev_id(resource: &str, serial: &str) -> String {
    format!("Dev{}", id_part(resource, serial))
}

fn id_part(resource: &str, serial: &str) -> String {
    let parts: Vec<&str> = resource.trim().split("::").collect();
    // A GPIB address, on a gateway or a local card: `gpib7,16[,1]`.
    if let Some(bus) = parts.iter().find_map(|p| gpib_digits(p)) {
        return bus;
    }
    let head = parts.first().copied().unwrap_or("");
    // A local GPIB card: `GPIB0::22[::1]::INSTR`.
    if let Some(board) = head.to_ascii_uppercase().strip_prefix("GPIB").filter(|b| b.bytes().all(|c| c.is_ascii_digit())) {
        let digits = |p: &&&str| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit());
        if let Some(primary) = parts.get(1).filter(digits) {
            let board = if board.is_empty() { "0" } else { board };
            let mut id = format!("{board}{primary}");
            if let Some(secondary) = parts.get(2).filter(digits) {
                id.push('-');
                id.push_str(secondary);
            }
            return id;
        }
    }
    // A USB instrument: `USB0::0x0957::0xFFEF::<serial>::INSTR` — its serial,
    // which USB carries in the address itself; vendor-product without one.
    if head.to_ascii_uppercase().starts_with("USB") && parts.len() >= 4 {
        let serial = if parts.len() >= 5 { parts[3] } else { "" };
        if !serial.trim().is_empty() {
            return segment(serial.trim());
        }
        return segment(&format!("{}-{}", parts[1], parts[2]));
    }
    // A serial port on this machine: the device node.
    if let Some(path) = head.strip_prefix("ASRL").or_else(|| head.strip_prefix("asrl")) {
        let node = path.rsplit('/').next().unwrap_or(path);
        if !node.is_empty() {
            return segment(node);
        }
    }
    // A LAN instrument: its serial, else its host.
    if head.to_ascii_uppercase().starts_with("TCPIP") {
        if let Some(serial) = usable_serial(serial) {
            return segment(&serial);
        }
        if let Some(host) = parts.get(1).filter(|h| !h.is_empty()) {
            return segment(&host.replace('.', "-"));
        }
    }
    segment(resource)
}

/// `gpib7,16` → `716`, `gpib7,30,1` → `730-1`; `None` for anything else.
fn gpib_digits(part: &str) -> Option<String> {
    let lower = part.to_ascii_lowercase();
    let rest = lower.strip_prefix("gpib").or_else(|| lower.strip_prefix("hpib"))?;
    let nums: Vec<&str> = rest.split(',').collect();
    if nums.len() < 2 || nums.iter().any(|n| n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let mut id = format!("{}{}", nums[0], nums[1]);
    if let Some(secondary) = nums.get(2) {
        id.push('-');
        id.push_str(secondary);
    }
    Some(id)
}

/// Safe as one topic segment and one container-name piece.
fn segment(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dev_id;

    #[test]
    fn gpib_is_board_and_primary() {
        assert_eq!(dev_id("TCPIP::44.44.44.222::gpib7,16::INSTR", "MY42000785"), "Dev716");
        assert_eq!(dev_id("TCPIP0::44.44.44.222::gpib7,6::INSTR", ""), "Dev76");
        // A local card spells the same address `GPIB<board>::<primary>[::<secondary>]`.
        assert_eq!(dev_id("GPIB0::22::INSTR", ""), "Dev022");
        assert_eq!(dev_id("GPIB0::30::1::INSTR", ""), "Dev030-1");
    }

    #[test]
    fn a_secondary_address_is_kept_apart() {
        assert_eq!(dev_id("TCPIP::44.44.44.222::gpib7,30,1::INSTR", ""), "Dev730-1");
        assert_ne!(dev_id("TCPIP::h::gpib7,1,11::INSTR", ""), dev_id("TCPIP::h::gpib7,11,1::INSTR", ""));
    }

    #[test]
    fn a_usb_instrument_is_its_serial() {
        assert_eq!(dev_id("USB0::0x0957::0xFFEF::CN03480580::INSTR", ""), "DevCN03480580");
        assert_eq!(dev_id("USB0::0x0957::0xFFEF::INSTR", ""), "Dev0x0957-0xFFEF");
    }

    #[test]
    fn a_serial_port_is_its_device_node() {
        assert_eq!(dev_id("ASRL/dev/ttyUSB0::INSTR", "0"), "DevttyUSB0");
    }

    #[test]
    fn a_lan_instrument_is_its_serial_else_its_host() {
        assert_eq!(dev_id("TCPIP::44.44.44.157::INSTR", "MY44044028"), "DevMY44044028");
        assert_eq!(dev_id("TCPIP0::44.44.44.157::5025::SOCKET", "MY44044028"), "DevMY44044028");
        assert_eq!(dev_id("TCPIP::44.44.44.157::INSTR", "0"), "Dev44-44-44-157");
    }
}
