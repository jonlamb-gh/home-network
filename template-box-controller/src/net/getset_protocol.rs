// TODO - move this into the params crate?
// or part of the controller lib crate

use crate::error::Error;
use crate::net::eth::MTU;
use log::{error, warn};
use params::{GetSetFrame, GetSetOp, GetSetPayloadType};

#[derive(Debug)]
pub struct GetSetProtocol<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> GetSetProtocol<T> {
    pub fn new(buffer: T) -> Result<GetSetProtocol<T>, Error> {
        if buffer.as_ref().len() < MTU {
            error!("GetSetProtocol needs a buffer with at least {} bytes", MTU);
            Err(Error::Capacity)
        } else {
            Ok(GetSetProtocol { buffer })
        }
    }

    pub fn buffer(&mut self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> GetSetProtocol<T> {
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut()
    }

    pub fn process_buffer<F>(&mut self, size: usize, mut handler: F) -> Result<(), Error>
    where
        F: FnMut(GetSetOp, &mut [u8]) -> Result<(), Error>,
    {
        let frame = GetSetFrame::new_checked(&self.buffer()[..size])?;
        let op = frame.op();

        // Attempt to catch malformed requests
        let malformed = match op {
            GetSetOp::ListAll => false,
            GetSetOp::Get => {
                if frame.payload_type() == GetSetPayloadType::ParameterIdListPacket {
                    false
                } else {
                    true
                }
            }
            GetSetOp::Set => {
                if frame.payload_type() == GetSetPayloadType::ParameterListPacket {
                    false
                } else {
                    true
                }
            }
        };

        if malformed {
            warn!("Got malformed request {}", frame);
            return Err(Error::ProtocolMalformed(op));
        }

        // User callback handling
        (handler)(op, self.buffer.as_mut())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use pretty_assertions::assert_eq;

    #[test]
    fn todo() {
        //
    }
}
