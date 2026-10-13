pub use crate::RpcResponseError;
use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, compact_format};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::RequestId;

/// Raw JSON-RPC envelope used on the wire.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use corsa_jsonrpc::{RawMessage, RequestId};
///
/// let message = RawMessage::request(RequestId::integer(1), "ping", json!({ "value": 1 }));
/// assert_eq!(message.method.as_deref(), Some("ping"));
/// assert_eq!(message.id, Some(RequestId::integer(1)));
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawMessage {
    /// JSON-RPC protocol version, normally `"2.0"`.
    #[serde(default = "jsonrpc_version")]
    pub jsonrpc: CompactString,
    /// Request/response identifier, absent for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Method name for requests and notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<CompactString>,
    /// Request or notification parameters.
    #[serde(default, deserialize_with = "deserialize_present_value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Successful response body.
    #[serde(default, deserialize_with = "deserialize_present_value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcResponseError>,
}

fn jsonrpc_version() -> CompactString {
    CompactString::from("2.0")
}

fn deserialize_present_value<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(
        Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null),
    ))
}

impl RawMessage {
    /// Builds a JSON-RPC request message.
    pub fn request(id: RequestId, method: impl Into<CompactString>, params: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Builds a JSON-RPC request without a `params` member.
    pub fn request_without_params(id: RequestId, method: impl Into<CompactString>) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: Some(method.into()),
            params: None,
            result: None,
            error: None,
        }
    }

    /// Builds a JSON-RPC notification message.
    pub fn notification(method: impl Into<CompactString>, params: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: None,
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    /// Builds a JSON-RPC notification without a `params` member.
    pub fn notification_without_params(method: impl Into<CompactString>) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: None,
            method: Some(method.into()),
            params: None,
            result: None,
            error: None,
        }
    }

    /// Builds a successful JSON-RPC response message.
    pub fn response(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Builds an error JSON-RPC response message.
    pub fn error(id: RequestId, error: RpcResponseError) -> Self {
        Self {
            jsonrpc: jsonrpc_version(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Classifies the envelope into a higher-level message kind.
    ///
    /// This clones the payload; prefer [`into_kind`](Self::into_kind) when the
    /// envelope is no longer needed afterwards.
    pub fn kind(&self) -> Result<MessageKind> {
        self.clone().into_kind()
    }

    /// Classifies the envelope into a higher-level message kind, consuming the
    /// envelope so payloads move instead of being deep-cloned.
    pub fn into_kind(self) -> Result<MessageKind> {
        if self.jsonrpc.as_str() != "2.0" {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "unsupported jsonrpc version: {}",
                self.jsonrpc
            ))));
        }
        match (self.id, self.method, self.result, self.error) {
            (Some(id), Some(method), None, None) => Ok(MessageKind::Request {
                id,
                method,
                params: self.params.unwrap_or(Value::Null),
            }),
            (None, Some(method), None, None) => Ok(MessageKind::Notification {
                method,
                params: self.params.unwrap_or(Value::Null),
            }),
            (Some(id), None, Some(result), None) => Ok(MessageKind::Response {
                id,
                result: Some(result),
                error: None,
            }),
            (Some(id), None, None, Some(error)) => Ok(MessageKind::Response {
                id,
                result: None,
                error: Some(error),
            }),
            (id, method, result, error) => {
                let message = RawMessage {
                    jsonrpc: self.jsonrpc,
                    id,
                    method,
                    params: self.params,
                    result,
                    error,
                };
                Err(CorsaError::UnexpectedMessage(compact_format(format_args!(
                    "{message:?}"
                ))))
            }
        }
    }
}

/// Parsed view of a [`RawMessage`].
#[derive(Clone, Debug)]
pub enum MessageKind {
    /// JSON-RPC request envelope.
    Request {
        /// Request identifier.
        id: RequestId,
        /// Method name.
        method: CompactString,
        /// Request parameters.
        params: Value,
    },
    /// JSON-RPC notification envelope.
    Notification {
        /// Method name.
        method: CompactString,
        /// Notification parameters.
        params: Value,
    },
    /// JSON-RPC response envelope.
    Response {
        /// Request identifier being answered.
        id: RequestId,
        /// Successful result body, when present.
        result: Option<Value>,
        /// Error body, when present.
        error: Option<RpcResponseError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_non_2_0_jsonrpc_version() {
        let message = RawMessage {
            jsonrpc: "1.0".into(),
            id: Some(RequestId::integer(1)),
            method: Some("ping".into()),
            params: None,
            result: None,
            error: None,
        };

        let error = message.kind().unwrap_err();
        assert!(matches!(
            error,
            CorsaError::Protocol(message) if message.contains("unsupported jsonrpc version")
        ));
    }

    #[test]
    fn rejects_request_like_message_with_response_fields() {
        let message = RawMessage {
            jsonrpc: "2.0".into(),
            id: Some(RequestId::integer(1)),
            method: Some("ping".into()),
            params: Some(json!({"value": 1})),
            result: Some(json!({"pong": true})),
            error: None,
        };

        assert!(matches!(
            message.kind().unwrap_err(),
            CorsaError::UnexpectedMessage(_)
        ));
    }

    #[test]
    fn rejects_response_with_both_result_and_error() {
        let message = RawMessage {
            jsonrpc: "2.0".into(),
            id: Some(RequestId::integer(1)),
            method: None,
            params: None,
            result: Some(json!({"pong": true})),
            error: Some(RpcResponseError {
                code: -32000,
                message: "broken".into(),
                data: None,
            }),
        };

        assert!(matches!(
            message.kind().unwrap_err(),
            CorsaError::UnexpectedMessage(_)
        ));
    }

    #[test]
    fn accepts_null_result_response() {
        let message: RawMessage =
            serde_json::from_value(json!({ "jsonrpc": "2.0", "id": 1, "result": null })).unwrap();

        assert!(matches!(
            message.kind().unwrap(),
            MessageKind::Response {
                id,
                result: Some(Value::Null),
                error: None,
            } if id == RequestId::integer(1)
        ));
    }

    #[test]
    fn params_less_messages_omit_params_instead_of_serializing_null() {
        let request = RawMessage::request_without_params(RequestId::integer(1), "shutdown");
        let notification = RawMessage::notification_without_params("exit");

        let request = serde_json::to_value(request).unwrap();
        let notification = serde_json::to_value(notification).unwrap();

        assert!(!request.as_object().unwrap().contains_key("params"));
        assert!(!notification.as_object().unwrap().contains_key("params"));
    }

    #[test]
    fn deserialization_preserves_null_params_member_presence() {
        let omitted: RawMessage = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "shutdown"
        }))
        .unwrap();
        let explicit_null: RawMessage = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null
        }))
        .unwrap();

        assert_eq!(omitted.params, None);
        assert_eq!(explicit_null.params, Some(Value::Null));
    }

    #[test]
    fn rejects_response_without_result_or_error() {
        let message: RawMessage =
            serde_json::from_value(json!({ "jsonrpc": "2.0", "id": 1 })).unwrap();

        assert!(matches!(
            message.kind().unwrap_err(),
            CorsaError::UnexpectedMessage(_)
        ));
    }
}
