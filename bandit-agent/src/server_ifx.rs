use std::net::TcpStream;
use std::io::{Read, Write};

pub struct ServerIfx {
    stream: TcpStream,
    read_buf: Vec<u8>,
}

impl ServerIfx {
    pub fn new(address: &str) -> ServerIfx {
        ServerIfx {
            stream: match TcpStream::connect(address) {
                Ok(stream) => {
                    let _ = stream.set_read_timeout(None);
                    stream
                },
                Err(_) => panic!("Could not connect to server!"),
            },
            read_buf: vec![0u8; 4],
        }
    }

    pub fn pull_arm(&mut self, arm: usize) -> u8 {
        let _ = self.stream.write(arm.to_string().as_bytes());
        if let Ok(len) = self.stream.read(self.read_buf.as_mut_slice()) {
            String::from_utf8(self.read_buf[..len-1].to_vec())
                .unwrap_or_else(|_| panic!("Bad stream!"))
                .parse::<u8>()
                .unwrap_or_else(|_| panic!("Bad reward!"))
        } else {
            panic!("Socket stream read failed!")
        }
    }
}
