//! 接口分类器
//!
//! 根据请求路径和 HTTP 方法对接口进行分类

use crate::rate_limit::types::EndpointType;
use axum::http::{Method, Uri};
use once_cell::sync::Lazy;
use regex::Regex;

/// 接口分类器
#[derive(Debug, Clone)]
pub struct EndpointClassifier;

/// 认证接口路径正则表达式
static AUTH_PATH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^/(api/)?(auth|login|register|refresh|logout)(/|$)").unwrap());

/// 管理接口路径正则表达式
static ADMIN_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^/(api/)?admin(/|$)").unwrap());

impl EndpointClassifier {
    pub fn new() -> Self {
        Self
    }

    /// 对请求进行分类
    ///
    /// # 分类规则
    /// 1. Auth: 认证相关接口（login, register, refresh, logout）
    /// 2. Admin: 管理接口（/api/admin/*）
    /// 3. Query: GET 请求（非 Auth/Admin）
    /// 4. Write: POST/PUT/DELETE/PATCH 请求（非 Auth/Admin）
    pub fn classify(&self, uri: &Uri, method: &Method) -> EndpointType {
        let path = uri.path();

        // 首先检查是否为认证接口
        if AUTH_PATH_REGEX.is_match(path) {
            return EndpointType::Auth;
        }

        // 检查是否为管理接口
        if ADMIN_PATH_REGEX.is_match(path) {
            return EndpointType::Admin;
        }

        // 根据 HTTP 方法分类
        match *method {
            Method::GET | Method::HEAD => EndpointType::Query,
            Method::POST | Method::PUT | Method::DELETE | Method::PATCH => EndpointType::Write,
            _ => EndpointType::Query, // 其他方法按 Query 处理
        }
    }
}

impl Default for EndpointClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_uri(path: &str) -> Uri {
        path.parse().unwrap()
    }

    #[test]
    fn test_classify_auth_endpoints() {
        let classifier = EndpointClassifier::new();

        // 标准 /api/auth/* 路径
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/login"), &Method::POST),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/register"), &Method::POST),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/refresh"), &Method::POST),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/logout"), &Method::POST),
            EndpointType::Auth
        );

        // 简化的 /auth/* 路径
        assert_eq!(
            classifier.classify(&create_uri("/auth/login"), &Method::POST),
            EndpointType::Auth
        );

        // 根路径的认证接口
        assert_eq!(
            classifier.classify(&create_uri("/login"), &Method::POST),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/register"), &Method::POST),
            EndpointType::Auth
        );
    }

    #[test]
    fn test_classify_admin_endpoints() {
        let classifier = EndpointClassifier::new();

        assert_eq!(
            classifier.classify(&create_uri("/api/admin/users"), &Method::GET),
            EndpointType::Admin
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/admin/settings"), &Method::POST),
            EndpointType::Admin
        );
        assert_eq!(
            classifier.classify(&create_uri("/admin/permissions"), &Method::DELETE),
            EndpointType::Admin
        );
    }

    #[test]
    fn test_classify_query_endpoints() {
        let classifier = EndpointClassifier::new();

        assert_eq!(
            classifier.classify(&create_uri("/api/users"), &Method::GET),
            EndpointType::Query
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/items/123"), &Method::GET),
            EndpointType::Query
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/dashboard"), &Method::HEAD),
            EndpointType::Query
        );
    }

    #[test]
    fn test_classify_write_endpoints() {
        let classifier = EndpointClassifier::new();

        assert_eq!(
            classifier.classify(&create_uri("/api/users"), &Method::POST),
            EndpointType::Write
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/items/123"), &Method::PUT),
            EndpointType::Write
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/items/123"), &Method::DELETE),
            EndpointType::Write
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/users/123"), &Method::PATCH),
            EndpointType::Write
        );
    }

    #[test]
    fn test_auth_overwrites_method() {
        let classifier = EndpointClassifier::new();

        // Auth 路径，无论什么方法都返回 Auth
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/login"), &Method::GET),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/me"), &Method::POST),
            EndpointType::Auth
        );
    }

    #[test]
    fn test_admin_overwrites_method() {
        let classifier = EndpointClassifier::new();

        // Admin 路径，无论什么方法都返回 Admin
        assert_eq!(
            classifier.classify(&create_uri("/api/admin/config"), &Method::POST),
            EndpointType::Admin
        );
    }

    #[test]
    fn test_classify_by_path_only() {
        let classifier = EndpointClassifier::new();

        assert_eq!(
            classifier.classify_by_path(&create_uri("/api/auth/login")),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify_by_path(&create_uri("/api/admin/users")),
            EndpointType::Admin
        );
        assert_eq!(
            classifier.classify_by_path(&create_uri("/api/users")),
            EndpointType::Query
        );
    }

    #[test]
    fn test_complex_paths() {
        let classifier = EndpointClassifier::new();

        // 包含查询字符串
        assert_eq!(
            classifier.classify(&"/api/users?page=1".parse().unwrap(), &Method::GET),
            EndpointType::Query
        );

        // 嵌套路径
        assert_eq!(
            classifier.classify(&create_uri("/api/auth/2fa/verify"), &Method::POST),
            EndpointType::Auth
        );
        assert_eq!(
            classifier.classify(&create_uri("/api/admin/users/123/roles"), &Method::GET),
            EndpointType::Admin
        );
    }

    #[test]
    fn test_edge_cases() {
        let classifier = EndpointClassifier::new();

        // 根路径
        assert_eq!(
            classifier.classify(&create_uri("/"), &Method::GET),
            EndpointType::Query
        );

        // 健康检查
        assert_eq!(
            classifier.classify(&create_uri("/health"), &Method::GET),
            EndpointType::Query
        );
    }
}
