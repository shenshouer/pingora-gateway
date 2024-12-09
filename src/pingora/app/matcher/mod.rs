#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::httproutes::HTTPRouteRulesBackendRefs;
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::httproutes::HTTPRouteRulesBackendRefs;

use http::request::Parts;

mod gateway;
mod http_route;
mod http_route_rule;
mod http_route_rule_match;

/// 匹配对应的HTTPRoute, 并返回HTTPRoute中合适的BackendRefs
pub trait HTTPRouteChecker {
    fn check_headers(&self, headers: &Parts, port: i32) -> Option<&Vec<HTTPRouteRulesBackendRefs>>;
}
