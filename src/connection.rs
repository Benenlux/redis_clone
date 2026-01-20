use std::net::TcpListener;

pub struct Connection {
    pub listener: TcpListener,
}

impl Connection {
    fn new(connection_uri: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(connection_uri)?;
        Ok(Self { listener })
    }
}
