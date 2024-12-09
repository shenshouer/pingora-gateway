use std::collections::HashMap;

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::httproutes::{
    HTTPRouteRulesMatches, HTTPRouteRulesMatchesHeaders, HTTPRouteRulesMatchesHeadersType,
    HTTPRouteRulesMatchesPathType, HTTPRouteRulesMatchesQueryParams,
    HTTPRouteRulesMatchesQueryParamsType,
};
#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::httproutes::{
    HTTPRouteRulesMatches, HTTPRouteRulesMatchesHeaders, HTTPRouteRulesMatchesHeadersType,
    HTTPRouteRulesMatchesPathType, HTTPRouteRulesMatchesQueryParams,
    HTTPRouteRulesMatchesQueryParamsType,
};

use http::request::Parts;

pub trait HTTPRouteRulesMatchesChecker {
    fn check_headers(&self, headers: &Parts) -> bool;
}

impl HTTPRouteRulesMatchesChecker for HTTPRouteRulesMatches {
    fn check_headers(&self, headers: &Parts) -> bool {
        // 1. 匹配path
        if !self.path.as_ref().map_or(true, |rule| {
            let path = headers.uri.path();
            match (rule.value.as_ref(), rule.r#type.as_ref()) {
                (Some(rule_path), None) => rule_path.starts_with(path),
                (Some(rule_path), Some(path_rule)) => match path_rule {
                    HTTPRouteRulesMatchesPathType::PathPrefix => rule_path.starts_with(path),
                    HTTPRouteRulesMatchesPathType::Exact => rule_path == path,
                    HTTPRouteRulesMatchesPathType::RegularExpression => {
                        regex::Regex::new(rule_path)
                            .map(|re| re.is_match(path))
                            .unwrap_or(false)
                    }
                },
                _ => true,
            }
        }) {
            return false;
        }
        // 2. 匹配header
        if !self.headers.as_ref().map_or(true, |header_rules| {
            header_rules.iter().all(|r| r.check_headers(headers))
        }) {
            return false;
        }
        // 3. 匹配 query
        if !self.query_params.as_ref().map_or(true, |query_rules| {
            query_rules.iter().all(|r| r.check_headers(headers))
        }) {
            return false;
        }

        // 4. 匹配methods
        if let Some(method) = self.method.as_ref() {
            let req_method = headers.method.as_ref().to_uppercase();
            let rule_method = serde_json::to_string(method).unwrap();
            if rule_method != req_method {
                return false;
            }
        }
        true
    }
}

impl HTTPRouteRulesMatchesChecker for HTTPRouteRulesMatchesHeaders {
    fn check_headers(&self, headers: &Parts) -> bool {
        match (
            headers
                .headers
                .get(&self.name)
                .and_then(|value| value.to_str().ok()),
            self.r#type.as_ref(),
        ) {
            // 如果没有规则时, 默认进行精确匹配
            (Some(actual_value), None) => *actual_value == self.value,
            (Some(actual_value), Some(typ)) => match typ {
                HTTPRouteRulesMatchesHeadersType::Exact => *actual_value == self.value,
                HTTPRouteRulesMatchesHeadersType::RegularExpression => {
                    regex::Regex::new(&self.value)
                        .map(|re| re.is_match(actual_value))
                        .unwrap_or(false)
                }
            },
            _ => false,
        }
    }
}

impl HTTPRouteRulesMatchesChecker for HTTPRouteRulesMatchesQueryParams {
    fn check_headers(&self, headers: &Parts) -> bool {
        let query_param_map = headers.uri.query().map(parse_query);
        match (query_param_map, self.r#type.clone()) {
            (Some(map), None) => map
                .get(&self.name)
                .map(|v| self.value == *v)
                .unwrap_or(false),
            (Some(map), Some(typ)) => match typ {
                HTTPRouteRulesMatchesQueryParamsType::Exact => map
                    .get(&self.name)
                    .map(|v| self.value == *v)
                    .unwrap_or(false),
                HTTPRouteRulesMatchesQueryParamsType::RegularExpression => map
                    .get(&self.name)
                    .map(|s| {
                        regex::Regex::new(&self.value)
                            .map(|re| re.is_match(s))
                            .ok()
                            .unwrap_or(false)
                    })
                    .unwrap_or(false),
            },
            _ => false,
        }
    }
}

/// 解析查询参数到Map,注意此处数组数据会被覆盖为k:v格式
/// TODO: 支持数组为 k: []
fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|s| s.split_once('=').map(|t| (t.0.to_owned(), t.1.to_owned())))
        .collect()
}
