// #[cfg(all(feature = "experimental", not(feature = "standard")))]
// use gateway_api::apis::experimental::{gateways::Gateway, httproutes::HTTPRoute};
// #[cfg(all(feature = "standard", not(feature = "experimental")))]
// use gateway_api::apis::standard::{gateways::Gateway, httproutes::HTTPRoute};

// use k8s_openapi::api::core::v1::Service;
// use kube::runtime::reflector::Store;

// pub struct KubeStore {
//     pub gateway: Store<Gateway>,
//     pub http_route: Store<HTTPRoute>,
//     pub service: Store<Service>,
// }

// pub struct KubeDiscovery {
//     pub store: KubeStore,
// }
