pub struct DefaultPort {
    pub p404: i32,
    pub p500: i32,
    pub tls_honeypot: i32,
}

pub(crate) const DEFAULT_PORT: DefaultPort = DefaultPort {
    p404: 60404,
    p500: 60500,
    tls_honeypot: 60443,
};
