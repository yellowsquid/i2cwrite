use embedded_hal::blocking::i2c::{self, Read};
use std::cmp;
use std::io::{self, Error};
use std::thread;
use std::time::Duration;

const BUFFER_WAIT: Duration = Duration::from_millis(4);

/// Encodes a single byte of data.
pub trait Encoding {
    fn encode(&self, byte: u8) -> Vec<u8>;
}

/// Wrapper trait for interoperability.
///
/// Reading from this returns the number of bytes free in the slave buffer. Writing sends data to
/// the slave. The slave will never have 0 bytes written.
pub trait ReadWrite: Read + i2c::Write {
    fn convert_read_err(error: <Self as Read>::Error) -> Error;
    fn convert_write_err(error: <Self as i2c::Write>::Error) -> Error;
}

/// Writes data to an I2C slave at a given address.
pub struct I2cWriter<'a, T: ReadWrite> {
    encoding: &'a dyn Encoding,
    device: &'a mut T,
    buffer: Vec<u8>,
    address: u8,
}

impl<'a, T: ReadWrite> io::Write for I2cWriter<'a, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut mapped = buf.iter().flat_map(|b| self.encoding.encode(*b)).collect();
        self.buffer.append(&mut mapped);

        let mut slave_buffer_size = [0];

        while !self.buffer.is_empty() {
            slave_buffer_size[0] = 0;
            self.device
                .read(self.address, &mut slave_buffer_size)
                .map_err(T::convert_read_err)?;

            if slave_buffer_size[0] == 0 {
                thread::sleep(BUFFER_WAIT);
            } else {
                let take = cmp::min(slave_buffer_size[0].into(), self.buffer.len());
                let remainder = self.buffer.split_off(take);
                self.device
                    .write(self.address, &self.buffer)
                    .map_err(T::convert_write_err)?;
                self.buffer = remainder;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, T: ReadWrite> I2cWriter<'a, T> {
    /// Creates a new item.
    pub fn new(encoding: &'a dyn Encoding, device: &'a mut T, address: u8) -> Self {
        Self {
            encoding,
            device,
            buffer: Vec::new(),
            address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::convert::Infallible;
    use std::io::Write;

    /// Identity encoding: what goes in comes out.
    struct Identity;

    impl Encoding for Identity {
        fn encode(&self, byte: u8) -> Vec<u8> {
            vec![byte]
        }
    }

    /// Expects a certain message and panics when something is wrong.
    struct ReadWriter {
        address: u8,
        expecting: Vec<u8>,
        buffer_cap: u8,
        buffer_limit: u8,
    }

    impl Read for ReadWriter {
        type Error = Infallible;

        /// Gets the number of bytes and increases fake buffer size if empty.
        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Infallible> {
            assert_eq!(address, self.address);
            assert_eq!(buffer.len(), 1);
            buffer[0] = self.buffer_cap;

            if self.buffer_cap == 0 {
                self.buffer_cap = self.buffer_limit;
            }

            Ok(())
        }
    }

    impl i2c::Write for ReadWriter {
        type Error = Infallible;

        /// Checks bytes match and decreases buffer size.
        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Infallible> {
            assert_eq!(address, self.address);
            assert!(bytes.len() <= self.buffer_cap.into());
            assert!(bytes.len() > 0);
            assert!(bytes.len() <= self.expecting.len());
            self.buffer_cap -= self.buffer_limit;

            for byte in bytes {
                let next = self.expecting.pop().unwrap();
                assert_eq!(*byte, next);
            }

            Ok(())
        }
    }

    impl ReadWrite for ReadWriter {
        /// Never fails so convert is unreachable.
        fn convert_read_err(_error: Infallible) -> Error {
            unreachable!()
        }

        /// Never fails so convert is unreachable.
        fn convert_write_err(_error: Infallible) -> Error {
            unreachable!()
        }
    }

    impl ReadWriter {
        /// Creates a new mock device.
        fn new(address: u8, expecting: &[u8], buffer_limit: u8) -> Self {
            let mut copy = Vec::with_capacity(expecting.len());
            copy.extend(expecting);
            copy.reverse();
            Self {
                address,
                expecting: copy,
                buffer_limit,
                buffer_cap: buffer_limit,
            }
        }

        fn is_done(&self) -> bool {
            self.expecting.is_empty()
        }
    }

    fn data_string() -> impl Strategy<Value = Vec<u8>> {
        prop::collection::vec(any::<u8>(), prop::collection::SizeRange::default())
    }

    #[test]
    fn write_zero() {
        let mut rw = ReadWriter::new(4, b"", 0);
        let mut writer = I2cWriter::new(&Identity, &mut rw, 4);
        let written = writer.write(b"").unwrap();
        assert_eq!(written, 0);
    }

    proptest! {
        #[test]
        fn write(data in data_string()) {
            let mut rw = ReadWriter::new(4, &data, 4);

            let mut writer = I2cWriter::new(&Identity, &mut rw, 4);
            let written = writer.write(&data).unwrap();
            assert_eq!(written, data.len());
            assert!(rw.is_done());
        }

        #[test]
        fn write_split(data in data_string()) {
            let mut rw = ReadWriter::new(4, &data, 4);

            let half = data.len() / 2;

            let mut writer = I2cWriter::new(&Identity, &mut rw, 4);
            let first_half = writer.write(&data[..half]).unwrap();
            assert_eq!(first_half, half);
            let second_half = writer.write(&data[half..]).unwrap();
            assert_eq!(second_half, data.len() - half);
            assert!(rw.is_done());
        }
    }
}
