//! copy from beta's
use core::str::FromStr;
use gdbstub::conn::{Connection, ConnectionExt};
use libax::io::{prelude::*, Error, Result};
use libax::net::{IpAddr, TcpListener, TcpStream};

const LOCAL_IP: &str = "10.0.2.15";

pub struct GdbServer {
    inner: TcpStream,
}

impl Connection for GdbServer {
    type Error = Error;

    fn write(&mut self, byte: u8) -> Result {
        Write::write_all(&mut self.inner, &[byte])
    }

    fn write_all(&mut self, buf: &[u8]) -> Result {
        Write::write_all(&mut self.inner, buf)
    }

    fn flush(&mut self) -> Result {
        Write::flush(&mut self.inner)
    }

    fn on_session_start(&mut self) -> Result {
        Ok(())
    }
}

impl ConnectionExt for GdbServer {
    fn read(&mut self) -> Result<u8> {
        let mut buf = [0u8];
        match Read::read_exact(&mut self.inner, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(None)
    }
}


impl GdbServer {
    pub fn new(port: u16) -> Result<Self> {
        let addr = IpAddr::from_str(LOCAL_IP).unwrap();
        let mut listener = TcpListener::bind((addr, port).into())?;
        println!("Waiting for a GDB connection on port {}...", port);
        let (stream, addr) = listener.accept()?;
        println!("Debugger connected from {}", addr);
        Ok(Self { inner: stream })
    }
}