use std::sync::{Arc, LazyLock};

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::{gateways::Gateway, httproutes::HTTPRoute};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::{gateways::Gateway, httproutes::HTTPRoute};

use ::pingora::server::Server;
use clap::Parser;
use config::Config;
use controller::{store::KubeStore, watch};
use k8s_openapi::api::core::v1::Service;
use kube::Client;
use tracing::info;

mod config;
mod controller;
mod pingora;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    server_config: String,
    #[clap(short, long, default_value = "pingora")]
    name: String,
    #[clap(short, long, default_value = "v0.1")]
    version: String,
}

pub static SHARED_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio shared runtime")
});

// pub static KUBE_CLIENT: LazyLock<Option<Client>> = LazyLock::new(|| {
//     let config = kube::Config::incluster()
//         .ok()
//         .or(SHARED_RUNTIME.block_on(kube::Config::infer()).ok())?;
//     let _guard = SHARED_RUNTIME.enter();
//     let client = Client::try_from(config).ok()?;
//     SHARED_RUNTIME
//         .block_on(async { controller::register(client.clone()).await })
//         .expect("Failed to register controller");
//     Some(client)
// });

pub static KUBE_STORE: LazyLock<Arc<KubeStore>> = LazyLock::new(|| {
    let config = kube::Config::incluster()
        .ok()
        .or(SHARED_RUNTIME.block_on(kube::Config::infer()).ok())
        .expect("Failed to infer kube config");

    let _guard = SHARED_RUNTIME.enter();
    let client = Client::try_from(config).expect("Failed to create kube client");

    SHARED_RUNTIME
        .block_on(async { controller::register(client.clone()).await })
        .expect("Failed to register controller");

    let (gw_store, gw_watch_stream) = watch::<Gateway>(client.clone());
    let (http_store, http_route_watch_stream) = watch::<HTTPRoute>(client.clone());
    let (svc_store, svc_watch_stream) = watch::<Service>(client.clone());

    SHARED_RUNTIME.spawn(async move {
        tokio::select! {
            _ = gw_watch_stream => info!("Gateway watch stream closed"),
            _ = http_route_watch_stream => info!("HTTPRoute watch stream closed"),
            _ = svc_watch_stream => info!("Service watch stream closed"),
        }
    });

    let store = KubeStore::new(gw_store, http_store, svc_store);
    Arc::new(store)
});

/// cargo r -- -s /Users/sope/workspaces/k8s/gateway/pingora-gateway/tests/server.yaml
fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    dbg!(&args);

    let server_config = Config::load_from_yaml(args.server_config)?;
    let opt = server_config.pingora_server_opt();
    let mut my_server = Server::new(opt)?;
    my_server.bootstrap();

    let ports = server_config.ports();
    let mut services = Vec::new();
    for port in ports {
        let svc = pingora::new_service(&my_server.configuration, port, KUBE_STORE.clone())?;
        services.push(svc);
    }
    my_server.add_services(services);

    my_server.run_forever()
}
