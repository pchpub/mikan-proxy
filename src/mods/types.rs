use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub http_port: u16,
    pub domain: String,
}
