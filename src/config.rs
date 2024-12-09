use anyhow::Result;
use pingora::{prelude::Opt, ErrorType::ReadError, OrErr};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::pingora::app;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    upgrade: bool,
    daemon: bool,
    nocapture: bool,
    test: bool,
    conf: String,
    ports: Vec<app::ServerPort>,
}

impl Config {
    pub fn load_from_yaml<P>(path: P) -> Result<Self>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        let conf_str = std::fs::read_to_string(&path).or_err_with(ReadError, || {
            format!("Unable to read conf file from {path}")
        })?;
        debug!("Conf file read from {path}");
        Ok(Self::from_yaml(&conf_str)?)
    }

    /// TODO: 转成自定义Error
    pub fn from_yaml(s: &str) -> serde_yaml::Result<Self> {
        serde_yaml::from_str::<Self>(s)
    }

    pub fn pingora_server_opt(&self) -> Opt {
        Opt {
            upgrade: self.upgrade,
            daemon: self.daemon,
            nocapture: self.nocapture,
            test: self.test,
            conf: Some(self.conf.clone()),
        }
    }

    pub fn ports(&self) -> Vec<app::ServerPort> {
        self.ports.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config_content = r#"
upgrade: false
daemon: false
nocapture: false
test: false
conf: /Users/sope/workspaces/k8s/gateway/gateway-pingora/tests/config.yaml
ports:
- port: 80
  protocol: HTTP
- port: 443
  protocol: HTTPS
  cert:
    crt: /Users/sope/workspaces/k8s/gateway/gateway-pingora/tests/keys/server.crt
    key: /Users/sope/workspaces/k8s/gateway/gateway-pingora/tests/keys/key.pem
- port: 8080
  protocol: TCP
- port: 8443
  protocol: TLS
  cert:
    crt: /Users/sope/workspaces/k8s/gateway/gateway-pingora/tests/keys/server.crt
    key: /Users/sope/workspaces/k8s/gateway/gateway-pingora/tests/keys/key.pem
- port: 53
  protocol: UDP
        "#;
        let config = Config::from_yaml(config_content).unwrap();
        dbg!(config);
    }
}
