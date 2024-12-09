use std::sync::Arc;

use async_trait::async_trait;
#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::httproutes::{HTTPRoute, HTTPRouteRulesBackendRefs};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::httproutes::{HTTPRoute, HTTPRouteRulesBackendRefs};

use http::request::Parts;
use kube::{runtime::reflector, ResourceExt};
use pingora::{
    lb::Backend,
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};

use crate::controller::store::KubeStore;

use super::{HTTPRouteChecker, ServerPort};

#[derive(Clone)]
pub struct HttpProxyApp {
    port: ServerPort,
    store: Arc<KubeStore>,
}

impl HttpProxyApp {
    pub fn new(port: ServerPort, store: Arc<KubeStore>) -> Self {
        Self { port, store }
    }

    fn matche_http_route(
        &self,
        headers: &Parts,
    ) -> pingora::Result<(Arc<HTTPRoute>, Vec<HTTPRouteRulesBackendRefs>)> {
        let http_routes_backends = self
            .store
            .http_route()
            .state()
            .iter()
            .filter_map(|r| {
                r.check_headers(headers, self.port.port)
                    .map(|backend_refs| (r.clone(), backend_refs.clone()))
            })
            .collect::<Vec<_>>();

        self.select_http_route(http_routes_backends)
    }

    fn select_http_route(
        &self,
        http_routes: Vec<(Arc<HTTPRoute>, Vec<HTTPRouteRulesBackendRefs>)>,
    ) -> pingora::Result<(Arc<HTTPRoute>, Vec<HTTPRouteRulesBackendRefs>)> {
        let http_routes = http_routes
            .into_iter()
            .filter_map(|(r, brs)| {
                let key = reflector::ObjectRef::new(&r.name_any()).within(&r.namespace().unwrap());
                match self.store.gateway().get(&key).is_some() {
                    true => Some((r, brs)),
                    false => None,
                }
            })
            .collect::<Vec<_>>();

        let http_route = http_routes
            .first()
            .ok_or(new_pingora_error("No suitable http route found"))?
            .clone();

        Ok(http_route)
    }

    fn route(&self, headers: &Parts) -> pingora::Result<Box<HttpPeer>> {
        // 第一步：找到匹配的http route, 可能有多个;
        // 第二步：从符合条件的gateway中再次过滤 http route, 可能有多个;
        // 第三步：选取一个http route, 从http route中获取backend ref
        // 此处需要结合LoadBanacer从多个service中获取cluster ip, port;
        let http_route = self.matche_http_route(headers)?;
        // let namespace = http_route.namespace().ok_or(new_pingora_error(
        //     "Namespace not specified for http route in kubernetes",
        // ))?;

        // let backend_ref = http_route
        //     .spec
        //     .rules
        //     .as_ref()
        //     .and_then(|rules| {
        //         rules.first().as_ref().and_then(|rule| {
        //             rule.backend_refs
        //                 .as_ref()
        //                 .and_then(|backend_refs| backend_refs.first())
        //         })
        //     })
        //     .ok_or(new_pingora_error(
        //         "No Backend refs found in kubernete http route",
        //     ))?;

        // let svc_name = &backend_ref.name;
        // let svc_port = backend_ref.port.ok_or(new_pingora_error(
        //     "No port found in  backend ref of http route",
        // ))?;

        // info!("service name: {svc_name}, port: {svc_port}");

        // self.store.service().state().iter().for_each(|s| {
        //     info!("service name: {}", s.name_any());
        // });

        // let key = reflector::ObjectRef::new(svc_name).within(&namespace);
        // let svc = self
        //     .store
        //     .service()
        //     .get(&key)
        //     .ok_or(new_pingora_error("No service found in kubernetes"))?;

        // info!(
        //     "service , clusterIp {:?} backend ref port {svc_port}",
        //     svc.spec.as_ref().unwrap().cluster_ip,
        // );

        // TODO: 此处为开发方便直接指定的为本地监听, 应该从kubernetes中获取
        // let backend = Backend::new(&format!(
        //     "{}:{svc_port}",
        //     svc.spec.as_ref().unwrap().cluster_ip.as_ref().unwrap()
        // ))?;
        let backend = Backend::new("127.0.0.1:80").unwrap();

        let peer = Box::new(HttpPeer::new(backend, false, "".to_string()));

        Ok(peer)
    }
}

#[async_trait]
impl ProxyHttp for HttpProxyApp {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut (),
    ) -> pingora::Result<Box<HttpPeer>> {
        let headers = session.req_header();
        self.route(headers)
    }
}

fn new_pingora_error(context: &'static str) -> pingora::Error {
    pingora::Error {
        etype: pingora::ErrorType::InvalidHTTPHeader,
        esource: pingora::ErrorSource::Downstream,
        retry: pingora::RetryType::Decided(false),
        cause: None,
        context: Some(pingora::ImmutStr::Static(context)),
    }
}
