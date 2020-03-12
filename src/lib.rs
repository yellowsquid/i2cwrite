use embedded_hal::blocking::i2c::{self, Read};
use std::io;

enum Encoding {
    Identity,
}

struct I2cWriter<T: Read + i2c::Write> {
    encoding: Encoding,
    writer: T,
}

impl<T: Read + i2c::Write> io::Write for I2cWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}
