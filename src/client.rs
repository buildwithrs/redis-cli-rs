use tokio::net::TcpStream;


#[derive(Debug)]
pub struct RedisClient {
    pub stream: TcpStream,
}

impl RedisClient {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}