use std::io::{self, BufRead, Read, Cursor, BufReader, Chain, Take};
use std::net::Shutdown;

use http::hyper::net::{HttpStream, NetworkStream};
use http::hyper::h1::HttpReader;

pub type StreamReader = HttpReader<HttpStream>;
pub type InnerStream = Chain<Take<Cursor<Vec<u8>>>, BufReader<StreamReader>>;

pub struct DataStream {
    pub stream: InnerStream,
    pub network: HttpStream,
}

impl Read for DataStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl BufRead for DataStream {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.stream.consume(amt)
    }
}

pub fn kill_stream<S: Read, N: NetworkStream>(stream: &mut S, network: &mut N) {
    io::copy(&mut stream.take(1024), &mut io::sink()).expect("sink");

    // If there are any more bytes, kill it.
    let mut buf = [0];
    if let Ok(n) = stream.read(&mut buf) {
        if n > 0 {
            warn_!("Data left unread. Force closing network stream.");
            if let Err(e) = network.close(Shutdown::Both) {
                error_!("Failed to close network stream: {:?}", e);
            }
        }
    }
}

impl Drop for DataStream {
    // Be a bad citizen and close the TCP stream if there's unread data.
    fn drop(&mut self) {
        kill_stream(&mut self.stream, &mut self.network);
    }
}

