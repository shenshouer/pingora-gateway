#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::{gateways::Gateway, httproutes::HTTPRoute};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::{gateways::Gateway, httproutes::HTTPRoute};

use k8s_openapi::api::core::v1::Service;
use kube::runtime::reflector::Store;

pub struct KubeStore {
    gateway: Store<Gateway>,
    http_route: Store<HTTPRoute>,
    service: Store<Service>,
}

impl KubeStore {
    pub fn new(
        gateway: Store<Gateway>,
        http_route: Store<HTTPRoute>,
        service: Store<Service>,
    ) -> Self {
        Self {
            gateway,
            http_route,
            service,
        }
    }

    pub fn gateway(&self) -> Store<Gateway> {
        self.gateway.clone()
    }

    pub fn http_route(&self) -> Store<HTTPRoute> {
        self.http_route.clone()
    }

    pub fn service(&self) -> Store<Service> {
        self.service.clone()
    }
}
