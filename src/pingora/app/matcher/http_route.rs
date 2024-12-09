use std::sync::Arc;

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::{
    gateways::Gateway,
    httproutes::{HTTPRoute, HTTPRouteRulesBackendRefs},
};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::{
    gateways::Gateway,
    httproutes::{HTTPRoute, HTTPRouteRulesBackendRefs},
};

use http::request::Parts;
use wildcard::Wildcard;

use crate::KUBE_STORE;

use super::{gateway::GatewayChecker, http_route_rule::HTTPRouteRulesChecker, HTTPRouteChecker};

fn gateways(port: i32) -> Vec<Arc<Gateway>> {
    KUBE_STORE
        .gateway()
        .state()
        .iter()
        .filter_map(|gw| {
            if gw.spec.listeners.iter().any(|l| l.port == port) {
                Some(gw.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

impl HTTPRouteChecker for Arc<HTTPRoute> {
    fn check_headers(&self, headers: &Parts, port: i32) -> Option<&Vec<HTTPRouteRulesBackendRefs>> {
        // 0. 检查是否有引用的上层, 当前为gateway
        self.spec.parent_refs.as_ref()?;

        // 1. 匹配hostnames
        let req_host = headers.headers.get("host")?;

        if !self.spec.hostnames.as_ref().map_or(false, |hostnames| {
            hostnames.iter().any(|s| {
                if s.starts_with("*.") {
                    let wildcard = Wildcard::new(s.as_bytes()).unwrap();
                    wildcard.is_match(req_host.as_bytes())
                } else {
                    req_host == s
                }
            })
        }) {
            return None;
        }

        // 2. 匹配gateway
        if !gateways(port).iter().any(|gw| self.check(gw)) {
            return None;
        }
        // 3. 匹配rules
        self.spec
            .rules
            .as_ref()
            .and_then(|rules| rules.iter().find_map(|rule| rule.check_headers(headers)))
    }
}
