// use std::sync::Arc;

// #[cfg(all(feature = "experimental", not(feature = "standard")))]
// use gateway_api::apis::experimental::{gateways::Gateway, httproutes::HTTPRoute};
// #[cfg(all(feature = "standard", not(feature = "experimental")))]
// use gateway_api::apis::standard::{gateways::Gateway, httproutes::HTTPRoute};

// use k8s_openapi::api::core::v1::Service;

mod crd;
mod event;
pub mod store;

pub use crd::register;
pub use event::watch;

// use pingora::{prelude::background_service, services::Service as PingoraService};
// use store::KubeStore;

// pub fn kube_background_services(
//     client: kube::Client,
// ) -> (Arc<KubeStore>, Vec<Box<dyn PingoraService>>) {
//     let (gw_store, gw_sender) = event::watch::<Gateway>(client.clone());
//     let (http_store, http_sender) = event::watch::<HTTPRoute>(client.clone());
//     let (svc_store, svc_sender) = event::watch::<Service>(client.clone());

//     let store = store::KubeStore::new(gw_store, http_store, svc_store);
//     let svcs: Vec<Box<dyn PingoraService>> = vec![
//         Box::new(background_service(
//             "Background Service for Gateway Event Watching",
//             gw_sender,
//         )),
//         Box::new(background_service(
//             "Background Service for HTTPRoute Event Watching",
//             http_sender,
//         )),
//         Box::new(background_service(
//             "Background Service for Service Event Watching",
//             svc_sender,
//         )),
//     ];
//     (Arc::new(store), svcs)
// }
