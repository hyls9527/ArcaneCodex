//! Log message sanitization to prevent sensitive data exposure. Redacts API keys, tokens, file paths, and URL query parameters from log output.
use regex::Regex;
use std::sync::OnceLock;

fn api_key_prefix() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?P<prefix>(?:Bearer\s+|sk[-_]|api_key\s*[:=]\s*|token\s*[:=]\s*|authorization\s*[:=]\s*|password\s*[:=]\s*|secret\s*[:=]\s*))(?P<key>[A-Za-z0-9_-]{8,64})").unwrap()
    })
}

fn encrypted_key() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(enc:v[12]:)([A-Za-z0-9+/=]{16,})").unwrap())
}

fn windows_path() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)[A-Za-z]:\\(?:[^\s:\\/]+[\\/])*[^\s:\\/]*\.\w+").unwrap())
}

fn unix_path() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)/(?:home|Users|tmp|var|opt|etc|srv)/(?:[^\s/]+)/[^\s]*").unwrap()
    })
}

fn url_sensitive_params() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let pattern = r"(?i)[?&](token|api_key|apikey|access_token|secret|password|auth|credential)[=][^&\s\x22\x27]*(?=[&\s\x22\x27]|$)";
        Regex::new(pattern).unwrap()
    })
}

/// Sanitizes a log message by redacting API keys, bearer tokens, encrypted keys, file paths (Windows and Unix), and sensitive URL query parameters. Returns the sanitized string.
pub fn sanitize_log_message(msg: &str) -> String {
    if msg.is_empty() || msg.len() < 4 {
        return msg.to_string();
    }

    let mut result = msg.to_string();

    result = api_key_prefix()
        .replace_all(&result, |caps: &regex::Captures| {
            let prefix = &caps["prefix"];
            let key = &caps["key"];
            let masked = mask_value(key);
            format!("{}{}", prefix, masked)
        })
        .to_string();

    result = encrypted_key()
        .replace_all(&result, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let encoded = &caps[2];
            let masked = mask_value(encoded);
            format!("{}{}", prefix, masked)
        })
        .to_string();

    result = windows_path()
        .replace_all(&result, "[REDACTED_PATH]")
        .to_string();

    result = unix_path()
        .replace_all(&result, "[REDACTED_PATH]")
        .to_string();

    result = url_sensitive_params()
        .replace_all(&result, |caps: &regex::Captures| {
            let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let param_name_end = full_match.find('=').unwrap_or(full_match.len());
            format!("{}=[REDACTED]", &full_match[..param_name_end])
        })
        .to_string();

    result
}

fn mask_value(value: &str) -> String {
    let len = value.len();
    if len <= 10 {
        "****".to_string()
    } else {
        format!("{}****{}", &value[..4], &value[len - 4..])
    }
}

#[allow(dead_code)]
/// Redacts an API key by masking the middle portion, keeping only first 4 and last 4 characters.
pub fn redact_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    mask_value(key)
}

#[allow(dead_code)]
/// Redacts a file path if it matches Windows or Unix path patterns, replacing it with [REDACTED_PATH].
pub fn redact_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    if windows_path().is_match(path) || unix_path().is_match(path) {
        return "[REDACTED_PATH]".to_string();
    }
    path.to_string()
}

#[allow(dead_code)]
/// Redacts sensitive query parameters from a URL by replacing values with [REDACTED].
pub fn redact_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }

    let sanitized = url_sensitive_params().replace_all(url, |caps: &regex::Captures| {
        let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let param_name_end = full_match.find('=').unwrap_or(full_match.len());
        format!("{}=[REDACTED]", &full_match[..param_name_end])
    });

    sanitized.to_string()
}

/// Initializes structured logging with log sanitization. Writes to APPDATA/ArcaneCodex/logs/app.log with INFO level and automatic sensitive data redaction.
pub fn init_sanitized_logging() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let log_dir = std::env::var("APPDATA")
        .map(|appdata| format!("{}\\ArcaneCodex\\logs", appdata))
        .unwrap_or_else(|_| "./logs".to_string());

    std::fs::create_dir_all(&log_dir).ok();

    let log_file = format!("{}\\app.log", log_dir);

    let file_writer = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file);

    match file_writer {
        Ok(_) => {
            let log_path = log_file.clone();
            let layer = fmt::layer()
                .with_writer(move || -> Box<dyn std::io::Write> {
                    match std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_path)
                    {
                        Ok(file) => Box::new(file),
                        Err(e) => {
                            eprintln!("Failed to open log file: {}", e);
                            Box::new(std::io::stdout())
                        }
                    }
                })
                .with_ansi(false)
                .event_format(SanitizedEventFormatter);

            tracing_subscriber::registry()
                .with(layer)
                .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
                .init();
        }
        Err(_) => {
            let layer = fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .event_format(SanitizedEventFormatter);

            tracing_subscriber::registry()
                .with(layer)
                .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
                .init();
        }
    }
}

/// Custom tracing event formatter that applies sanitize_log_message to all log output before writing.
struct SanitizedEventFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for SanitizedEventFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> Result<(), std::fmt::Error> {
        let mut message = String::new();
        event.record(&mut MessageVisitor {
            message: &mut message,
        });
        let sanitized = sanitize_log_message(&message);
        write!(writer, "{}", sanitized)
    }
}

struct MessageVisitor<'a> {
    message: &'a mut String,
}

impl tracing::field::Visit for MessageVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            *self.message = value.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_short_key() {
        assert_eq!(mask_value("short"), "****");
    }

    #[test]
    fn test_mask_long_key() {
        assert_eq!(mask_value("abcdefghij"), "abcd****ghij");
    }

    #[test]
    fn test_sanitize_sk_prefix() {
        let msg = "API Key: sk-proj-AbCdEfGhIjKlMnOpQrStUvWxYz";
        let sanitized = sanitize_log_message(msg);
        assert!(
            !sanitized.contains("AbCdEfGhIjKlMnOpQrStUvWxYz"),
            "Full key should be masked"
        );
        assert!(sanitized.contains("sk-proj-"));
        assert!(sanitized.contains("****"));
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let msg = "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(sanitized.contains("Bearer "));
    }

    #[test]
    fn test_sanitize_encrypted_v2_key() {
        let msg = "Stored key: enc:v2:UGFyYW5vaWRfZGF0YV9oZXJlX2luX2Jhc2U2NA==";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("UGFyYW5vaWRfZGF0YV9oZXJlX2luX2Jhc2U2NA=="));
        assert!(sanitized.contains("enc:v2:"));
    }

    #[test]
    fn test_sanitize_windows_path() {
        let msg = "File loaded from C:\\Users\\hyls9527\\Pictures\\vacation.jpg";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("C:\\Users\\hyls9527"));
        assert!(sanitized.contains("[REDACTED_PATH]"));
    }

    #[test]
    fn test_sanitize_unix_path() {
        let msg = "Reading /home/user/photos/private/image.png";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("/home/user"));
        assert!(sanitized.contains("[REDACTED_PATH]"));
    }

    #[test]
    fn test_sanitize_url_query_params() {
        let msg = "Request URL: https://api.example.com/v1/chat?token=secret123&model=gpt-4";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("secret123"));
        assert!(sanitized.contains("token=[REDACTED]"));
        assert!(sanitized.contains("model=gpt-4"));
    }

    #[test]
    fn test_safe_message_unchanged() {
        let msg = "AI task queue started with 3 workers";
        assert_eq!(sanitize_log_message(msg), msg);
    }

    #[test]
    fn test_empty_and_short_messages() {
        assert_eq!(sanitize_log_message(""), "");
        assert_eq!(sanitize_log_message("ok"), "ok");
    }

    #[test]
    fn test_redact_api_key_function() {
        assert_eq!(redact_api_key(""), "");
        assert_eq!(redact_api_key("short"), "****");
        assert_eq!(redact_api_key("my-long-api-key-value-here"), "my-l****here");
    }

    #[test]
    fn test_redact_path_function() {
        assert_eq!(redact_path(""), "");
        assert_eq!(redact_path("/some/random/path"), "/some/random/path");
        assert_eq!(
            redact_path("C:\\Users\\admin\\secrets.txt"),
            "[REDACTED_PATH]"
        );
    }

    #[test]
    fn test_redact_url_function() {
        assert_eq!(redact_url(""), "");
        let url = "https://api.example.com/data?page=1&limit=20";
        assert_eq!(redact_url(url), url);
        let sensitive_url = "https://api.example.com/auth?access_token=tok_12345";
        let redacted = redact_url(sensitive_url);
        assert!(!redacted.contains("tok_12345"));
        assert!(redacted.contains("access_token=[REDACTED]"));
    }

    #[test]
    fn test_no_regex_catastrophic_backtracking() {
        let long_junk = "x".repeat(10000);
        let msg = format!("Processing: {}", long_junk);
        let start = std::time::Instant::now();
        let _ = sanitize_log_message(&msg);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 50,
            "Sanitization took too long: {:?}",
            elapsed
        );
    }
}
