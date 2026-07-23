//! Telnet receive buffer: assembles complete lines from mud telnet data events.
//!
//! Lines are delimited by CRLF on incoming data payloads, or by a Go-Ahead (GA) IAC
//! command flushing buffered bytes without a trailing CRLF.
//!
//! **UTF-8 policy:** Each completed line is decoded as UTF-8. Lines that are not
//! valid UTF-8 are skipped (same effective behavior as before); a `debug!` log is
//! emitted when a line is dropped. Lossy decoding and surfacing decode errors to
//! callers are out of scope.

use bytes::{BufMut, BytesMut};
use libmudtelnet::events::TelnetEvents;
use libmudtelnet::telnet::op_command;
use log::debug;
use std::io::BufRead;

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
                if !bytes.ends_with(CARRIAGE_RETURN_NEW_LINE) {
                    self.buffer.put(bytes.clone());
                    return Vec::new();
                }

                let mut buffer = std::mem::replace(&mut self.buffer, BytesMut::with_capacity(1024));
                buffer.put(bytes.clone());

                return self.process_input_data(buffer);
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
        for line_result in bytes.lines() {
            match line_result {
                Ok(line) => lines.push(line),
                Err(err) => debug!("Skipping telnet line: {err}"),
            }
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
    fn invalid_utf8_line_is_skipped_without_panic() {
        let mut buffer = TelnetBuffer::new();

        let lines = buffer.handle_event(&TelnetEvents::DataReceive(Bytes::from_static(
            b"valid\r\n\xff\r\nalso valid\r\n",
        )));

        assert_eq!(lines, vec!["valid".to_string(), "also valid".to_string()]);
    }
}
