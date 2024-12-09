#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::httproutes::{HTTPRouteRules, HTTPRouteRulesBackendRefs};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::httproutes::{HTTPRouteRules, HTTPRouteRulesBackendRefs};

use http::request::Parts;

use super::http_route_rule_match::HTTPRouteRulesMatchesChecker;

pub trait HTTPRouteRulesChecker {
    fn check_headers(&self, headers: &Parts) -> Option<&Vec<HTTPRouteRulesBackendRefs>>;
}

impl HTTPRouteRulesChecker for HTTPRouteRules {
    fn check_headers(&self, headers: &Parts) -> Option<&Vec<HTTPRouteRulesBackendRefs>> {
        // 有backend再谈其他的
        let is_match = self.backend_refs.is_some()
            && self
                .matches
                .iter()
                .any(|matches| matches.iter().any(|m| m.check_headers(headers)));

        match is_match {
            true => self.backend_refs.as_ref(),
            false => None,
        }
    }
}
