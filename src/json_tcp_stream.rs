use std::io::Read;
use std::net::TcpStream;

/// A non-blocking stream that reads JSON objects using serde.
pub struct JsonTcpStream {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl JsonTcpStream {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).expect("set stream non blocking");
        Self { stream, buffer: vec![] }
    }

    pub fn read<T: serde::de::DeserializeOwned>(&mut self) -> Option<T> {
        let mut read_buf = [0; 1024];
        let size = self.stream.read(&mut read_buf).ok()?;
        if size == 0 {
            return None;
        }
        let read_bytes = &read_buf[..size];
        self.buffer.extend_from_slice(read_bytes);

        match serde_json::from_slice(self.buffer.as_slice()) {
            Ok(result) => {
                // Clear the buffer since the data has been read successfully.
                self.buffer.clear();
                result
            }
            Err(_err) => {
                // println!("Partial data: {:?} err = {:?}", String::from_utf8(self.buffer.clone()), err);
                // If this fails to read we assume that is because we received a partial part of
                // the data. Since it was added to the buffer we can continue trying next time
                // this method is called.
                return None;
            }
        }
    }

    pub fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
}