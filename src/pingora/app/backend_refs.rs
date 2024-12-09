use std::sync::Arc;

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::httproutes::HTTPRouteRulesBackendRefs;
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::httproutes::HTTPRouteRulesBackendRefs;

use k8s_openapi::api::core::v1::Service;
use kube::runtime::reflector::{self, Store};
use pingora::lb::Backend;

pub struct HTTPRouteRulesBackends {
    store: Arc<Store<Service>>,
    backend_refs: Vec<HTTPRouteRulesBackendRefs>,
}

impl HTTPRouteRulesBackends {
    fn to_pingora_backends(&self) {}
}

trait IntoPingoraBackend {
    fn into_pingora_backend(self, store: Arc<Store<Service>>, namespace: &str) -> Option<Backend>;
}

impl IntoPingoraBackend for HTTPRouteRulesBackendRefs {
    fn into_pingora_backend(self, store: Arc<Store<Service>>, namespace: &str) -> Option<Backend> {
        // 只匹配 Service 类型的后端
        if self.kind.as_ref().map_or(true, |kind| kind != "Service") {
            return None;
        }
        let key = reflector::ObjectRef::new(&self.name)
            .within(self.namespace.as_deref().unwrap_or(namespace));
        let svc = store.get(&key)?;
        let cluster_ip = svc.spec.as_ref()?.cluster_ip.clone()?;
        // 如果没有指定端口，且 Service 只有一个端口，则使用该端口
        // 如果没有指定端口，且 Service 有多个端口，则返回 None
        // 如果指定了端口，则使用指定的端口
        // Backend {}
        todo!()
    }
}
