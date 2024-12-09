use std::sync::Arc;

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::{
    gateways::Gateway,
    httproutes::{HTTPRoute, HTTPRouteParentRefs},
};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::{
    gateways::Gateway,
    httproutes::{HTTPRoute, HTTPRouteParentRefs},
};
use kube::ResourceExt;

pub trait GatewayChecker {
    fn check(&self, gateway: &Arc<Gateway>) -> bool;
}

/// 参考: https://gateway-api.sigs.k8s.io/reference/spec/#gateway.networking.k8s.io/v1.CommonRouteSpec
impl GatewayChecker for Arc<HTTPRoute> {
    fn check(&self, gateway: &Arc<Gateway>) -> bool {
        self.spec.parent_refs.as_ref().map_or(false, |parent_refs| {
            parent_refs
                .iter()
                .any(|parent_ref| parent_ref.check(gateway))
        })
    }
}

const GROUP_GATEWAY: &str = "gateway.networking.k8s.io";
const KIND_GATEWAY: &str = "Gateway";

impl GatewayChecker for HTTPRouteParentRefs {
    fn check(&self, gateway: &Arc<Gateway>) -> bool {
        // 匹配端口
        if !match (self.port, self.section_name.as_ref()) {
            (None, Some(section_name)) => gateway
                .spec
                .listeners
                .iter()
                .any(|l| l.name == *section_name),
            (Some(ref_port), None) => gateway.spec.listeners.iter().any(|l| l.port == ref_port),
            (Some(ref_port), Some(section_name)) => gateway
                .spec
                .listeners
                .iter()
                .any(|l| l.port == ref_port && l.name == *section_name),
            (None, None) => true,
        } {
            return false;
        }

        if !self
            .group
            .as_ref()
            .map_or(true, |g| g.is_empty() || g == GROUP_GATEWAY)
        {
            return false;
        }

        if !self.kind.as_ref().map_or(true, |k| k == KIND_GATEWAY) {
            return false;
        }

        if !self
            .namespace
            .as_ref()
            .map_or(false, |ns| *ns == gateway.namespace().unwrap())
        {
            return false;
        }

        if self.name != gateway.name_any() {
            return false;
        }
        true
    }
}
