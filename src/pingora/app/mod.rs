use std::fmt::Display;

use matcher::HTTPRouteChecker;
use serde::{Deserialize, Serialize};

mod backend_refs;
pub mod http;
mod matcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Protocol {
    TLS,
    HTTPS,
    HTTP,
    TCP,
    UDP,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::TLS => "TLS",
                Protocol::HTTPS => "HTTPS",
                Protocol::HTTP => "HTTP",
                Protocol::TCP => "TCP",
                Protocol::UDP => "UDP",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Certificate {
    pub crt: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPort {
    port: i32,
    name: Option<String>,
    pub protocol: Protocol,
    pub cert: Option<Certificate>,
}

impl ServerPort {
    pub fn addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
