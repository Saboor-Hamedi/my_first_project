use crate::model::{Category, Finding, Severity};
use reqwest::header::HeaderMap;

pub fn check(headers: &HeaderMap, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    for val in headers.get_all(reqwest::header::SET_COOKIE) {
        let cookie_str = match val.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let parts: Vec<&str> = cookie_str.split(';').map(|p| p.trim()).collect();
        if parts.is_empty() {
            continue;
        }

        let cookie_name = parts[0].split('=').next().unwrap_or("unknown");
        let lower = cookie_str.to_lowercase();

        // 1. HttpOnly flag check
        if !lower.contains("httponly") {
            findings.push(Finding {
                category: Category::Cookies,
                severity: Severity::Medium,
                title: format!("Cookie '{}' missing HttpOnly flag", cookie_name),
                description: format!("Cookie '{}' can be accessed by client-side JavaScript, increasing XSS session theft risk.", cookie_name),
            });
        }

        // 2. Secure flag check
        if is_https && !lower.contains("secure") {
            findings.push(Finding {
                category: Category::Cookies,
                severity: Severity::High,
                title: format!("Cookie '{}' missing Secure flag", cookie_name),
                description: format!("Cookie '{}' transmitted over HTTPS lacks the Secure flag and may leak over unencrypted HTTP.", cookie_name),
            });
        }

        // 3. SameSite flag check
        if !lower.contains("samesite") {
            findings.push(Finding {
                category: Category::Cookies,
                severity: Severity::Low,
                title: format!("Cookie '{}' missing SameSite attribute", cookie_name),
                description: format!("Cookie '{}' does not specify SameSite (Lax/Strict), increasing CSRF vulnerability.", cookie_name),
            });
        } else if lower.contains("samesite=none") && !lower.contains("secure") {
            findings.push(Finding {
                category: Category::Cookies,
                severity: Severity::High,
                title: format!("Cookie '{}' has SameSite=None without Secure", cookie_name),
                description: "SameSite=None must be paired with Secure flag to prevent cookie rejection and leakage.".into(),
            });
        }
    }

    findings
}
