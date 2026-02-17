use serde_json::json;
use std::fmt;
use std::time::Duration;

use crate::rules::llm_rules::LlmResponse;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL: &str = "claude-opus-4-5-20251101";
const MAX_TOKENS: u32 = 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const TOOL_NAME: &str = "report_lint_violations";

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
    ) -> impl std::future::Future<Output = Result<LlmResponse, LlmError>> + Send;
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

fn extract_tool_input_from_response(body: &serde_json::Value) -> Result<LlmResponse, LlmError> {
    let content = body["content"]
        .as_array()
        .ok_or_else(|| LlmError::ParseError("missing content array in API response".into()))?;

    let tool_block = content
        .iter()
        .find(|block| block["type"].as_str() == Some("tool_use"))
        .ok_or_else(|| LlmError::ParseError("no tool_use block in API response".into()))?;

    serde_json::from_value(tool_block["input"].clone())
        .map_err(|e| LlmError::ParseError(format!("failed to parse tool input: {e}")))
}

impl LlmAnalyzer for AnthropicClient {
    async fn analyze(
        &self,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<LlmResponse, LlmError> {
        let body = json!({
            "model": MODEL,
            "max_tokens": MAX_TOKENS,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
                }
            ],
            "tools": [{
                "name": TOOL_NAME,
                "description": "Report lint violations found in the file",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "has_violations": { "type": "boolean" },
                        "violations": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "text": { "type": "string", "description": "The offending text" },
                                    "reason": { "type": "string", "description": "Why this is a violation" }
                                },
                                "required": ["text", "reason"]
                            }
                        }
                    },
                    "required": ["has_violations", "violations"]
                }
            }],
            "tool_choice": { "type": "tool", "name": TOOL_NAME }
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

        extract_tool_input_from_response(&response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_body_includes_tools_and_tool_choice() {
        let system_prompt = "You are a linter.";
        let user_content = "Check this file content.";

        let body = json!({
            "model": MODEL,
            "max_tokens": MAX_TOKENS,
            "system": system_prompt,
            "messages": [{"role": "user", "content": user_content}],
            "tools": [{
                "name": TOOL_NAME,
                "description": "Report lint violations found in the file",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "has_violations": { "type": "boolean" },
                        "violations": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "text": { "type": "string", "description": "The offending text" },
                                    "reason": { "type": "string", "description": "Why this is a violation" }
                                },
                                "required": ["text", "reason"]
                            }
                        }
                    },
                    "required": ["has_violations", "violations"]
                }
            }],
            "tool_choice": { "type": "tool", "name": TOOL_NAME }
        });

        assert_eq!(body["model"], "claude-opus-4-5-20251101");
        assert_eq!(body["tools"][0]["name"], "report_lint_violations");
        assert_eq!(body["tool_choice"]["type"], "tool");
        assert_eq!(body["tool_choice"]["name"], "report_lint_violations");
    }

    #[test]
    fn extracts_violations_from_tool_use_response() {
        let response_body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "tool_use",
                    "id": "toolu_abc",
                    "name": "report_lint_violations",
                    "input": {
                        "has_violations": true,
                        "violations": [
                            {"text": "some bad text", "reason": "it is bad"}
                        ]
                    }
                }
            ],
            "model": "claude-opus-4-5-20251101",
            "stop_reason": "tool_use"
        });

        let result = extract_tool_input_from_response(&response_body).unwrap();
        assert!(result.has_violations);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].text, "some bad text");
        assert_eq!(result.violations[0].reason, "it is bad");
    }

    #[test]
    fn extracts_no_violations_from_tool_use_response() {
        let response_body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "tool_use",
                    "id": "toolu_abc",
                    "name": "report_lint_violations",
                    "input": {
                        "has_violations": false,
                        "violations": []
                    }
                }
            ],
            "stop_reason": "tool_use"
        });

        let result = extract_tool_input_from_response(&response_body).unwrap();
        assert!(!result.has_violations);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn returns_error_for_missing_tool_use_block() {
        let response_body = json!({
            "content": [
                {"type": "text", "text": "some text response"}
            ]
        });

        let result = extract_tool_input_from_response(&response_body);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_empty_content_array() {
        let response_body = json!({"content": []});

        let result = extract_tool_input_from_response(&response_body);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_missing_content_field() {
        let response_body = json!({"id": "msg_123", "type": "error"});

        let result = extract_tool_input_from_response(&response_body);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_malformed_tool_input() {
        let response_body = json!({
            "content": [{
                "type": "tool_use",
                "id": "toolu_abc",
                "name": "report_lint_violations",
                "input": {
                    "has_violations": "not_a_bool",
                    "violations": []
                }
            }]
        });

        let result = extract_tool_input_from_response(&response_body);
        assert!(result.is_err());
    }

    #[test]
    fn extracts_tool_use_when_mixed_with_text_blocks() {
        let response_body = json!({
            "content": [
                {"type": "text", "text": "I'll check..."},
                {
                    "type": "tool_use",
                    "id": "toolu_abc",
                    "name": "report_lint_violations",
                    "input": {
                        "has_violations": true,
                        "violations": [
                            {"text": "offending", "reason": "why"}
                        ]
                    }
                }
            ]
        });

        let result = extract_tool_input_from_response(&response_body).unwrap();
        assert!(result.has_violations);
        assert_eq!(result.violations.len(), 1);
    }
}
