// #[cfg(all(feature = "experimental", not(feature = "standard")))]
// use gateway_api::apis::experimental::{gateways::Gateway, httproutes::HTTPRoute};
// #[cfg(all(feature = "standard", not(feature = "experimental")))]
// use gateway_api::apis::standard::{gateways::Gateway, httproutes::HTTPRoute};

// use k8s_openapi::api::core::v1::Service;
// use kube::Client;

// use super::{event::watch, store::KubeStore};

// pub async fn run(client: Client, name: &str) -> anyhow::Result<()> {
//     let (gw_store, gw_watcher_stream) = watch::<Gateway>(client.clone());
//     let (http_store, http_watcher_stream) = watch::<HTTPRoute>(client.clone());
//     let (svc_store, svc_watcher_stream) = watch::<Service>(client.clone());

//     let store = KubeStore::new(gw_store, http_store, svc_store);

//     Ok(())
// }
