use core::convert::Infallible;
use embedded_hal_nb::serial::{ErrorType, Read};

/// Mock UART for test code.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MockUart {
    data: [u8; 64],
    read_pos: usize,
    write_pos: usize,
}

impl Default for MockUart {
    fn default() -> Self {
        Self::new()
    }
}

impl MockUart {
    pub fn new() -> Self {
        Self { data: [0; 64], read_pos: 0, write_pos: 0 }
    }
    /// Helper to load the mock with test data (eg an SBUS packet).
    pub fn push_data(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.data[self.write_pos] = b;
            self.write_pos += 1;
        }
    }
}

impl ErrorType for MockUart {
    type Error = Infallible;
}

impl Read<u8> for MockUart {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.read_pos < self.write_pos {
            let byte = self.data[self.read_pos];
            self.read_pos += 1;
            Ok(byte)
        } else {
            // Signal that there is no more data currently available
            Err(nb::Error::WouldBlock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbus_parsing() {
        let mut mock = MockUart::new();

        // A valid SBUS packet (simplified example)
        let sbus_packet = [0x0F, 0x00, 0x00, /* ... 22 bytes ... */ 0x00];
        mock.push_data(&sbus_packet);

        /*let mut rx = SbusReceiver::new(mock);

        // Call update and check if it correctly parsed the frame
        let result = rx.update(1000);

        assert!(result.is_ok());
        let frame = result.unwrap().expect("Should have parsed a frame");
        assert_eq!(frame.status, LinkStatus::Ok);*/
    }
}
