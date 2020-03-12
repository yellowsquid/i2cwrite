use embedded_hal::blocking::i2c::{self as hal, Read};
use i2cwrite::{Encoding, I2cWriter, ReadWrite};
use rppal::i2c::{self, I2c};
use std::io::{self, ErrorKind};
use std::sync::Mutex;

fn main() {
    let mut input = io::stdin();
    let mut rw = WrapI2c::new(I2c::new().expect("cannot find i2c device"));
    let mut output = I2cWriter::new(&Identity, &mut rw, 4);
    io::copy(&mut input, &mut output).expect("failed to write data");
}

struct WrapI2c {
    lock: Mutex<I2c>,
}

impl Read for WrapI2c {
    type Error = i2c::Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> i2c::Result<()> {
        let mut i2c = self.lock.lock().unwrap();
        i2c.set_slave_address(address.into())?;

        let mut count = 0;
        while count < buffer.len() {
            let new = i2c.read(&mut buffer[count..])?;
            count += new;
        }

        Ok(())
    }
}

impl hal::Write for WrapI2c {
    type Error = i2c::Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> i2c::Result<()> {
        let mut i2c = self.lock.lock().unwrap();
        i2c.set_slave_address(address.into())?;

        let mut count = 0;
        while count < bytes.len() {
            let new = i2c.write(&bytes[count..])?;
            count += new;
        }

        Ok(())
    }
}

impl ReadWrite for WrapI2c {
    fn convert_read_err(error: i2c::Error) -> io::Error {
        match error {
            i2c::Error::Io(error) => error,
            e => io::Error::new(ErrorKind::Other, e),
        }
    }

    fn convert_write_err(error: i2c::Error) -> io::Error {
        Self::convert_read_err(error)
    }
}

impl WrapI2c {
    fn new(i2c: I2c) -> Self {
        Self {
            lock: Mutex::new(i2c),
        }
    }
}

struct Identity;

impl Encoding for Identity {
    fn encode(&self, byte: u8) -> Vec<u8> {
        vec![byte]
    }
}
