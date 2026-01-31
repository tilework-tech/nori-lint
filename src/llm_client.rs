use serde_json::json;
use std::fmt;
use std::time::Duration;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-sonnet-4-20250514";
const MAX_TOKENS: u32 = 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub enum LlmError {
    HttpError(String),
    ParseError(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            LlmError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

pub trait LlmAnalyzer: Send + Sync {
    fn analyze(
        &self,
        system_prompt: &str,
        user_content: &str,
    ) -> impl std::future::Future<Output = Result<String, LlmError>> + Send;
}

pub struct AnthropicClient {
    api_key: String,
    http: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");
        Self { api_key, http }
    }
}

fn extract_text_from_response(body: &serde_json::Value) -> Result<String, LlmError> {
    body["content"]
        .get(0)
        .and_then(|block| block["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            LlmError::ParseError(format!(
                "No text content in API response: {}",
                serde_json::to_string(body).unwrap_or_default()
            ))
        })
}

impl LlmAnalyzer for AnthropicClient {
    async fn analyze(&self, system_prompt: &str, user_content: &str) -> Result<String, LlmError> {
        let body = json!({
            "model": MODEL,
            "max_tokens": MAX_TOKENS,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
                }
            ]
        });

        let response = self
            .http
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(LlmError::HttpError(format!(
                "API returned status {status}: {body_text}"
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        extract_text_from_response(&response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_correct_request_body() {
        let system_prompt = "You are a linter.";
        let user_content = "Check this file content.";

        let body = json!({
            "model": MODEL,
            "max_tokens": MAX_TOKENS,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
                }
            ]
        });

        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["system"], system_prompt);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], user_content);
    }

    #[test]
    fn parses_successful_api_response() {
        let response_body: serde_json::Value = serde_json::from_str(
            r#"{
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [
                    {
                        "type": "text",
                        "text": "{\"has_violations\": false, \"explanations\": []}"
                    }
                ],
                "model": "claude-sonnet-4-20250514",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 100, "output_tokens": 50}
            }"#,
        )
        .unwrap();

        let text = extract_text_from_response(&response_body).unwrap();
        assert_eq!(text, r#"{"has_violations": false, "explanations": []}"#);
    }

    #[test]
    fn returns_error_for_empty_content_array() {
        let response_body: serde_json::Value = serde_json::from_str(r#"{"content": []}"#).unwrap();

        let result = extract_text_from_response(&response_body);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_missing_content_field() {
        let response_body: serde_json::Value =
            serde_json::from_str(r#"{"id": "msg_123", "type": "error"}"#).unwrap();

        let result = extract_text_from_response(&response_body);
        assert!(result.is_err());
    }
}
