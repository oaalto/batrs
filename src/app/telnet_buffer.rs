//! Telnet receive buffer: assembles complete lines from mud telnet data events.
//!
//! Lines are delimited by CRLF on incoming data payloads, or by a Go-Ahead (GA) IAC
//! command flushing buffered bytes without a trailing CRLF. Completed lines are
//! released immediately; only the incomplete suffix remains buffered.
//!
//! **UTF-8 policy:** Each completed line is decoded as UTF-8. Lines that are not
//! valid UTF-8 are decoded lossily (`\u{FFFD}` replacement) rather than dropped,
//! so a single high-bit byte in a mud line does not lose the whole line.

use bytes::{BufMut, BytesMut};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use log::debug;

static CARRIAGE_RETURN_NEW_LINE: &[u8] = &[13, 10];

pub struct TelnetBuffer {
    buffer: BytesMut,
}

impl TelnetBuffer {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub fn handle_event(&mut self, event: &TelnetEvents) -> Vec<String> {
        match event {
            TelnetEvents::IAC(iac) => {
                debug!("IAC: {iac:?}");
                if op_command::GA == iac.command {
                    let buffer = std::mem::replace(&mut self.buffer, BytesMut::with_capacity(1024));
                    return self.process_input_data(buffer);
                }
            }
            TelnetEvents::Negotiation(neg) => {
                debug!("Negotiation: {neg:?}");
            }
            TelnetEvents::Subnegotiation(sub_neg) => {
                debug!("Subnegotiation: {sub_neg:?}");
            }
            TelnetEvents::DataReceive(bytes) => {
                self.buffer.put(bytes.clone());
                let Some(end) = self
                    .buffer
                    .windows(CARRIAGE_RETURN_NEW_LINE.len())
                    .rposition(|pair| pair == CARRIAGE_RETURN_NEW_LINE)
                else {
                    return Vec::new();
                };
                let completed = self.buffer.split_to(end + CARRIAGE_RETURN_NEW_LINE.len());
                return self.process_input_data(completed);
            }
            TelnetEvents::DataSend(_) => {}
            TelnetEvents::DecompressImmediate(_) => {
                debug!("Decompress data");
            }
        }

        Vec::new()
    }

    fn process_input_data(&self, bytes: BytesMut) -> Vec<String> {
        let mut lines = Vec::new();
        // CRLF-delimited; split on LF and then strip a trailing CR from each line.
        // Each line is decoded lossily so a single high-bit byte (common in mud map
        // rows) does not drop the whole line along with its @ player marker.
        for raw in bytes[..].split(|&b| b == b'\n') {
            let line: &[u8] = if raw.ends_with(b"\r") {
                &raw[..raw.len() - 1]
            } else {
                raw
            };
            lines.push(String::from_utf8_lossy(line).into_owned());
        }
        // match BufRead::lines(): no trailing empty line for a final newline
        if lines.last() == Some(&String::new()) {
            lines.pop();
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use libmudtelnet::events::TelnetIAC;

    #[test]
    fn crlf_delimited_input_yields_expected_lines() {
        let mut buffer = TelnetBuffer::new();

        let lines = buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
            b"first line\r\nsecond line\r\n",
        )));

        assert_eq!(
            lines,
            vec!["first line".to_string(), "second line".to_string()]
        );
    }

    #[test]
    fn ga_flushes_buffered_bytes_without_trailing_crlf() {
        let mut buffer = TelnetBuffer::new();

        assert!(
            buffer
                .handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
                    b"buffered line"
                )))
                .is_empty()
        );

        let lines = buffer.handle_event(&TelnetEvents::IAC(TelnetIAC::new(op_command::GA)));

        assert_eq!(lines, vec!["buffered line".to_string()]);
    }

    #[test]
    fn complete_lines_are_released_before_a_partial_prompt() {
        let mut buffer = TelnetBuffer::new();
        assert_eq!(
            buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
                b"response\r\npartial prompt"
            ))),
            vec!["response"]
        );
        assert_eq!(
            buffer.handle_event(&TelnetEvents::IAC(TelnetIAC::new(op_command::GA))),
            vec!["partial prompt"]
        );
    }

    #[test]
    fn crlf_split_between_events_releases_only_complete_lines() {
        let mut buffer = TelnetBuffer::new();
        assert!(
            buffer
                .handle_event(&TelnetEvents::DataReceive(Bytes::from_static(b"first\r")))
                .is_empty()
        );
        assert_eq!(
            buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
                b"\nsecond\r\nthird\r"
            ))),
            vec!["first", "second"]
        );
        assert_eq!(
            buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(b"\n"))),
            vec!["third"]
        );
    }

    #[test]
    fn invalid_utf8_line_is_decoded_lossily_instead_of_dropped() {
        let mut buffer = TelnetBuffer::new();

        let lines = buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
            b"valid\r\n\xff\r\nalso valid\r\n",
        )));

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "valid");
        // the invalid byte is replaced, but the whole line survives
        assert_eq!(lines[1].chars().collect::<Vec<_>>(), vec!['\u{FFFD}']);
        assert_eq!(lines[2], "also valid");
    }
}
