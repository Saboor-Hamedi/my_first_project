use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Headers,
    Cookies,
    Tls,
    Disclosure,
    Injection,
    Outdated,
}

impl Category {
    pub fn title(&self) -> &'static str {
        match self {
            Category::Headers => "Security Headers",
            Category::Cookies => "Cookie Security",
            Category::Tls => "TLS & Transport",
            Category::Disclosure => "Information Disclosure",
            Category::Injection => "Injection Indicators",
            Category::Outdated => "Fingerprint & Outdated Tech",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: Category,
    pub severity: Severity,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsInfo {
    pub protocol: Option<String>,
    pub cipher: Option<String>,
    pub issuer: Option<String>,
    pub expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub url: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub tls: Option<TlsInfo>,
    pub server_header: Option<String>,
    pub page_size_bytes: usize,
    pub note: Option<String>,
    pub findings: Vec<Finding>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_serde_roundtrip() {
        let result = ScanResult {
            url: "https://example.com".into(),
            status_code: 200,
            response_time_ms: 150,
            tls: Some(TlsInfo {
                protocol: Some("TLSv1.3".into()),
                cipher: Some("TLS_AES_256_GCM_SHA384".into()),
                issuer: Some("DigiCert".into()),
                expiry: Some("2027-01-01".into()),
            }),
            server_header: Some("nginx/1.18.0".into()),
            page_size_bytes: 1256,
            note: Some("Testing scan".into()),
            findings: vec![Finding {
                category: Category::Headers,
                severity: Severity::Medium,
                title: "Missing CSP".into(),
                description: "No Content-Security-Policy header found".into(),
            }],
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: ScanResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.url, "https://example.com");
        assert_eq!(deserialized.status_code, 200);
        assert_eq!(deserialized.findings.len(), 1);
        assert_eq!(deserialized.findings[0].severity, Severity::Medium);
    }
}
