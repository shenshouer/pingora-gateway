use std::sync::Arc;

use app::{http, Protocol, ServerPort};
use pingora::{proxy::http_proxy_service, server::configuration::ServerConf, services};

use crate::controller::store::KubeStore;

pub mod app;
mod discovery;

pub fn new_service(
    conf: &Arc<ServerConf>,
    port: ServerPort,
    store: Arc<KubeStore>,
) -> anyhow::Result<Box<dyn services::Service>> {
    let protocol = port.protocol.clone();
    let addr = port.addr();
    let cert = port.cert.clone();
    let mut svc = match protocol {
        Protocol::HTTPS | Protocol::HTTP => {
            let proxy_http = http::HttpProxyApp::new(port, store);
            http_proxy_service(conf, proxy_http)
        }
        Protocol::TCP | Protocol::TLS => unimplemented!("TCP/TLS is not supported yet"),
        Protocol::UDP => unimplemented!("UDP is not supported yet"),
    };

    if let Some(cert) = cert {
        svc.add_tls(&addr, &cert.crt, &cert.key)?;
    } else {
        svc.add_tcp(&addr);
    }
    Ok(Box::new(svc))
}
