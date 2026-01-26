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

                let mut buffer =
                    std::mem::replace(&mut self.buffer, BytesMut::with_capacity(1024));
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

    #[allow(clippy::lines_filter_map_ok)]
    fn process_input_data(&self, bytes: BytesMut) -> Vec<String> {
        bytes.lines().filter_map(Result::ok).collect()
    }
}
