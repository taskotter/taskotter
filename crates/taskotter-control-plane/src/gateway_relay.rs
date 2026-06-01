use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayStreamRelayInput {
    pub request_id: String,
    pub correlation_id: String,
    pub sequence: u32,
    pub provider: String,
    pub model: String,
    pub event: GatewayNormalizedStreamEvent,
    #[serde(default)]
    pub route: Option<GatewayRouteEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GatewayNormalizedStreamEvent {
    Started,
    OutputDelta {
        text: String,
    },
    ToolCallDelta {
        tool_call_id: String,
        name: String,
        arguments_delta: String,
    },
    UsageEstimate {
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        total_tokens: Option<u32>,
        estimated_cost_micros: Option<u64>,
    },
    Completed {
        finish_reason: GatewayFinishReason,
    },
    Cancelled {
        reason_code: GatewayTerminalReasonCode,
        safe_message: String,
    },
    Failed {
        reason_code: GatewayTerminalReasonCode,
        safe_message: String,
        retryable: bool,
        upstream_status: Option<u16>,
    },
    Denied {
        reason_code: GatewayTerminalReasonCode,
        safe_message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewayFinishReason {
    Stop,
    Length,
    ToolCall,
    Refusal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewayTerminalReasonCode {
    ClientCancelled,
    PolicyDenied,
    UsageLimitReached,
    ProviderTimeout,
    ProviderRateLimited,
    ProviderUnavailable,
    MalformedProviderStream,
    InternalGatewayError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GatewayRouteEvidence {
    pub route_type: GatewayRouteType,
    pub reason_code: GatewayRoutingReasonCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_decision_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewayRouteType {
    Primary,
    Fallback,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewayRoutingReasonCode {
    ExplicitSelection,
    PolicyDefault,
    CapabilityMatch,
    CostLimit,
    LatencyPreference,
    ResidencyConstraint,
    RunnerLocalRequired,
    FallbackAfterError,
    FallbackAfterCapacity,
    PolicyDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientSafeGatewayRelayEvent {
    pub version: String,
    #[serde(rename = "type")]
    pub event_type: ClientSafeGatewayRelayEventType,
    pub request_id: String,
    pub correlation_id: String,
    pub sequence: u32,
    pub provider: String,
    pub model: String,
    pub redaction: GatewayRelayRedaction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<GatewayRouteEvidence>,
    pub payload: ClientSafeGatewayRelayPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum ClientSafeGatewayRelayEventType {
    #[serde(rename = "gateway.relay.started")]
    Started,
    #[serde(rename = "gateway.relay.output_delta")]
    OutputDelta,
    #[serde(rename = "gateway.relay.tool_call_delta")]
    ToolCallDelta,
    #[serde(rename = "gateway.relay.usage_estimate")]
    UsageEstimate,
    #[serde(rename = "gateway.relay.completed")]
    Completed,
    #[serde(rename = "gateway.relay.cancelled")]
    Cancelled,
    #[serde(rename = "gateway.relay.failed")]
    Failed,
    #[serde(rename = "gateway.relay.denied")]
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GatewayRelayRedaction {
    ClientSafe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientSafeGatewayRelayPayload {
    Empty,
    OutputDelta {
        text: String,
    },
    ToolCallDelta {
        tool_call_id: String,
        name: String,
        arguments_delta: String,
    },
    UsageEstimate {
        input_tokens: Option<u32>,
        output_tokens: Option<u32>,
        total_tokens: Option<u32>,
        estimated_cost_micros: Option<u64>,
    },
    Completed {
        finish_reason: GatewayFinishReason,
    },
    Terminal {
        reason_code: GatewayTerminalReasonCode,
        safe_message: String,
        retryable: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        upstream_status: Option<u16>,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GatewayRelayMappingError {
    #[error("{0} is required")]
    Required(&'static str),
    #[error("{field} contains a sensitive pattern")]
    SensitivePattern { field: &'static str },
    #[error("{field} must use an allowed opaque reference prefix")]
    InvalidReference { field: &'static str },
    #[error("sequence must start at 0")]
    InvalidSequence,
    #[error("upstream_status must be in the HTTP status code range")]
    InvalidUpstreamStatus,
}

impl GatewayStreamRelayInput {
    pub fn into_client_safe_event(
        self,
    ) -> Result<ClientSafeGatewayRelayEvent, GatewayRelayMappingError> {
        require_opaque_ref("request_id", &self.request_id, &["req_"])?;
        require_opaque_ref("correlation_id", &self.correlation_id, &["corr_"])?;
        require_non_empty("provider", &self.provider)?;
        require_non_empty("model", &self.model)?;
        if let Some(route) = &self.route {
            route.validate()?;
        }

        if self.sequence == u32::MAX {
            return Err(GatewayRelayMappingError::InvalidSequence);
        }

        let (event_type, payload) = match self.event {
            GatewayNormalizedStreamEvent::Started => (
                ClientSafeGatewayRelayEventType::Started,
                ClientSafeGatewayRelayPayload::Empty,
            ),
            GatewayNormalizedStreamEvent::OutputDelta { text } => {
                require_client_safe_text("text", &text)?;
                (
                    ClientSafeGatewayRelayEventType::OutputDelta,
                    ClientSafeGatewayRelayPayload::OutputDelta { text },
                )
            }
            GatewayNormalizedStreamEvent::ToolCallDelta {
                tool_call_id,
                name,
                arguments_delta,
            } => {
                require_non_empty("tool_call_id", &tool_call_id)?;
                require_non_empty("name", &name)?;
                require_client_safe_text("arguments_delta", &arguments_delta)?;
                (
                    ClientSafeGatewayRelayEventType::ToolCallDelta,
                    ClientSafeGatewayRelayPayload::ToolCallDelta {
                        tool_call_id,
                        name,
                        arguments_delta,
                    },
                )
            }
            GatewayNormalizedStreamEvent::UsageEstimate {
                input_tokens,
                output_tokens,
                total_tokens,
                estimated_cost_micros,
            } => (
                ClientSafeGatewayRelayEventType::UsageEstimate,
                ClientSafeGatewayRelayPayload::UsageEstimate {
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    estimated_cost_micros,
                },
            ),
            GatewayNormalizedStreamEvent::Completed { finish_reason } => (
                ClientSafeGatewayRelayEventType::Completed,
                ClientSafeGatewayRelayPayload::Completed { finish_reason },
            ),
            GatewayNormalizedStreamEvent::Cancelled {
                reason_code,
                safe_message,
            } => terminal_event(
                ClientSafeGatewayRelayEventType::Cancelled,
                reason_code,
                safe_message,
                false,
                None,
            )?,
            GatewayNormalizedStreamEvent::Failed {
                reason_code,
                safe_message,
                retryable,
                upstream_status,
            } => terminal_event(
                ClientSafeGatewayRelayEventType::Failed,
                reason_code,
                safe_message,
                retryable,
                upstream_status,
            )?,
            GatewayNormalizedStreamEvent::Denied {
                reason_code,
                safe_message,
            } => terminal_event(
                ClientSafeGatewayRelayEventType::Denied,
                reason_code,
                safe_message,
                false,
                None,
            )?,
        };

        Ok(ClientSafeGatewayRelayEvent {
            version: "0.1.0".to_owned(),
            event_type,
            request_id: self.request_id,
            correlation_id: self.correlation_id,
            sequence: self.sequence,
            provider: self.provider,
            model: self.model,
            redaction: GatewayRelayRedaction::ClientSafe,
            route: self.route,
            payload,
        })
    }
}

impl GatewayRouteEvidence {
    fn validate(&self) -> Result<(), GatewayRelayMappingError> {
        if let Some(policy_decision_id) = &self.policy_decision_id {
            require_opaque_ref("policy_decision_id", policy_decision_id, &["poldec_"])?;
        }
        Ok(())
    }
}

fn terminal_event(
    event_type: ClientSafeGatewayRelayEventType,
    reason_code: GatewayTerminalReasonCode,
    safe_message: String,
    retryable: bool,
    upstream_status: Option<u16>,
) -> Result<
    (
        ClientSafeGatewayRelayEventType,
        ClientSafeGatewayRelayPayload,
    ),
    GatewayRelayMappingError,
> {
    require_client_safe_text("safe_message", &safe_message)?;
    if let Some(status) = upstream_status
        && !(100..=599).contains(&status)
    {
        return Err(GatewayRelayMappingError::InvalidUpstreamStatus);
    }

    Ok((
        event_type,
        ClientSafeGatewayRelayPayload::Terminal {
            reason_code,
            safe_message,
            retryable,
            upstream_status,
        },
    ))
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), GatewayRelayMappingError> {
    if value.trim().is_empty() {
        Err(GatewayRelayMappingError::Required(field))
    } else {
        require_client_safe_text(field, value)
    }
}

fn require_opaque_ref(
    field: &'static str,
    value: &str,
    allowed_prefixes: &[&str],
) -> Result<(), GatewayRelayMappingError> {
    require_non_empty(field, value)?;
    if allowed_prefixes
        .iter()
        .any(|prefix| value.starts_with(prefix))
    {
        Ok(())
    } else {
        Err(GatewayRelayMappingError::InvalidReference { field })
    }
}

fn require_client_safe_text(
    field: &'static str,
    value: &str,
) -> Result<(), GatewayRelayMappingError> {
    let lowered = value.to_ascii_lowercase();
    let sensitive = [
        "authorization:",
        "bearer ",
        "api_key",
        "access_token",
        "refresh_token",
        "credential",
        "secret",
        "client_secret",
        "private_key",
        "raw_prompt",
        "raw_log",
        "raw_payload",
        "artifact_body",
        "password",
        "cookie",
        "set-cookie",
        "-----begin",
    ];

    if sensitive.iter().any(|pattern| lowered.contains(pattern)) {
        Err(GatewayRelayMappingError::SensitivePattern { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn base_input(event: GatewayNormalizedStreamEvent, sequence: u32) -> GatewayStreamRelayInput {
        GatewayStreamRelayInput {
            request_id: "req_gateway_stream_1".to_owned(),
            correlation_id: "corr_gateway_stream_1".to_owned(),
            sequence,
            provider: "openai-compatible".to_owned(),
            model: "test-model".to_owned(),
            event,
            route: Some(GatewayRouteEvidence {
                route_type: GatewayRouteType::Fallback,
                reason_code: GatewayRoutingReasonCode::FallbackAfterError,
                policy_decision_id: Some("poldec_gateway_stream_1".to_owned()),
            }),
        }
    }

    fn relay_fixture_events() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(include_str!(
            "../../../contracts/fixtures/gateway-relay-events.json"
        ))?)
    }

    #[test]
    fn maps_all_gateway_stream_states_to_client_safe_events()
    -> Result<(), Box<dyn std::error::Error>> {
        let inputs = [
            (
                GatewayNormalizedStreamEvent::Started,
                ClientSafeGatewayRelayEventType::Started,
            ),
            (
                GatewayNormalizedStreamEvent::OutputDelta {
                    text: "hello".to_owned(),
                },
                ClientSafeGatewayRelayEventType::OutputDelta,
            ),
            (
                GatewayNormalizedStreamEvent::ToolCallDelta {
                    tool_call_id: "call_1".to_owned(),
                    name: "lookup".to_owned(),
                    arguments_delta: "{\"query\":\"status\"}".to_owned(),
                },
                ClientSafeGatewayRelayEventType::ToolCallDelta,
            ),
            (
                GatewayNormalizedStreamEvent::UsageEstimate {
                    input_tokens: Some(10),
                    output_tokens: Some(3),
                    total_tokens: Some(13),
                    estimated_cost_micros: Some(42),
                },
                ClientSafeGatewayRelayEventType::UsageEstimate,
            ),
            (
                GatewayNormalizedStreamEvent::Completed {
                    finish_reason: GatewayFinishReason::Stop,
                },
                ClientSafeGatewayRelayEventType::Completed,
            ),
            (
                GatewayNormalizedStreamEvent::Cancelled {
                    reason_code: GatewayTerminalReasonCode::ClientCancelled,
                    safe_message: "Request cancelled by the user.".to_owned(),
                },
                ClientSafeGatewayRelayEventType::Cancelled,
            ),
            (
                GatewayNormalizedStreamEvent::Failed {
                    reason_code: GatewayTerminalReasonCode::ProviderRateLimited,
                    safe_message: "Provider rate limit reached.".to_owned(),
                    retryable: true,
                    upstream_status: Some(429),
                },
                ClientSafeGatewayRelayEventType::Failed,
            ),
            (
                GatewayNormalizedStreamEvent::Denied {
                    reason_code: GatewayTerminalReasonCode::PolicyDenied,
                    safe_message: "Request denied by policy.".to_owned(),
                },
                ClientSafeGatewayRelayEventType::Denied,
            ),
        ];

        for (index, (event, expected_type)) in inputs.into_iter().enumerate() {
            let relay = base_input(event, index as u32).into_client_safe_event()?;
            assert_eq!(relay.version, "0.1.0");
            assert_eq!(relay.event_type, expected_type);
            assert_eq!(relay.redaction, GatewayRelayRedaction::ClientSafe);
            assert!(relay.route.is_some());
        }

        Ok(())
    }

    #[test]
    fn mapper_output_serializes_to_gateway_relay_contract_fixture()
    -> Result<(), Box<dyn std::error::Error>> {
        let inputs = [
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_1".to_owned(),
                correlation_id: "corr_gateway_stream_1".to_owned(),
                sequence: 0,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::Started,
                route: Some(GatewayRouteEvidence {
                    route_type: GatewayRouteType::Primary,
                    reason_code: GatewayRoutingReasonCode::CapabilityMatch,
                    policy_decision_id: Some("poldec_gateway_stream_1".to_owned()),
                }),
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_1".to_owned(),
                correlation_id: "corr_gateway_stream_1".to_owned(),
                sequence: 1,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::OutputDelta {
                    text: "Hello".to_owned(),
                },
                route: None,
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_1".to_owned(),
                correlation_id: "corr_gateway_stream_1".to_owned(),
                sequence: 2,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::ToolCallDelta {
                    tool_call_id: "call_1".to_owned(),
                    name: "lookup_status".to_owned(),
                    arguments_delta: "{\"issue\":\"BOG-538\"}".to_owned(),
                },
                route: None,
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_1".to_owned(),
                correlation_id: "corr_gateway_stream_1".to_owned(),
                sequence: 3,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::UsageEstimate {
                    input_tokens: Some(20),
                    output_tokens: Some(4),
                    total_tokens: Some(24),
                    estimated_cost_micros: Some(30),
                },
                route: None,
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_1".to_owned(),
                correlation_id: "corr_gateway_stream_1".to_owned(),
                sequence: 4,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::Completed {
                    finish_reason: GatewayFinishReason::Stop,
                },
                route: None,
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_2".to_owned(),
                correlation_id: "corr_gateway_stream_2".to_owned(),
                sequence: 0,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::Cancelled {
                    reason_code: GatewayTerminalReasonCode::ClientCancelled,
                    safe_message: "Request cancelled by the user.".to_owned(),
                },
                route: None,
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_3".to_owned(),
                correlation_id: "corr_gateway_stream_3".to_owned(),
                sequence: 0,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::Failed {
                    reason_code: GatewayTerminalReasonCode::ProviderRateLimited,
                    safe_message: "Provider rate limit reached.".to_owned(),
                    retryable: true,
                    upstream_status: Some(429),
                },
                route: Some(GatewayRouteEvidence {
                    route_type: GatewayRouteType::Fallback,
                    reason_code: GatewayRoutingReasonCode::FallbackAfterError,
                    policy_decision_id: Some("poldec_gateway_stream_3".to_owned()),
                }),
            },
            GatewayStreamRelayInput {
                request_id: "req_gateway_stream_4".to_owned(),
                correlation_id: "corr_gateway_stream_4".to_owned(),
                sequence: 0,
                provider: "openai-compatible".to_owned(),
                model: "test-model".to_owned(),
                event: GatewayNormalizedStreamEvent::Denied {
                    reason_code: GatewayTerminalReasonCode::PolicyDenied,
                    safe_message: "Request denied by policy.".to_owned(),
                },
                route: Some(GatewayRouteEvidence {
                    route_type: GatewayRouteType::Denied,
                    reason_code: GatewayRoutingReasonCode::PolicyDenied,
                    policy_decision_id: Some("poldec_gateway_stream_4".to_owned()),
                }),
            },
        ];

        let mapped = inputs
            .into_iter()
            .map(|input| {
                input.into_client_safe_event().and_then(|event| {
                    serde_json::to_value(event)
                        .map_err(|_| GatewayRelayMappingError::Required("serialized_event"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(mapped, relay_fixture_events()?);
        assert_eq!(mapped[0]["type"], json!("gateway.relay.started"));
        assert_eq!(mapped[7]["route"]["route_type"], json!("denied"));
        assert!(mapped[0].get("event_type").is_none());
        Ok(())
    }

    #[test]
    fn rejects_sensitive_provider_metadata_before_relay() {
        let input = base_input(
            GatewayNormalizedStreamEvent::Failed {
                reason_code: GatewayTerminalReasonCode::InternalGatewayError,
                safe_message: "Authorization: Bearer token leaked".to_owned(),
                retryable: false,
                upstream_status: Some(500),
            },
            0,
        );

        assert_eq!(
            input.into_client_safe_event(),
            Err(GatewayRelayMappingError::SensitivePattern {
                field: "safe_message"
            })
        );
    }

    #[test]
    fn rejects_schema_sensitive_patterns_and_invalid_route_refs() {
        for safe_message in [
            "password leaked",
            "artifact_body leaked",
            "raw_payload leaked",
            "credential leaked",
            "client secret leaked",
            "Set-Cookie leaked",
        ] {
            let input = base_input(
                GatewayNormalizedStreamEvent::Failed {
                    reason_code: GatewayTerminalReasonCode::InternalGatewayError,
                    safe_message: safe_message.to_owned(),
                    retryable: false,
                    upstream_status: Some(500),
                },
                0,
            );

            assert_eq!(
                input.into_client_safe_event(),
                Err(GatewayRelayMappingError::SensitivePattern {
                    field: "safe_message"
                })
            );
        }

        let mut input = base_input(GatewayNormalizedStreamEvent::Started, 0);
        input.route = Some(GatewayRouteEvidence {
            route_type: GatewayRouteType::Denied,
            reason_code: GatewayRoutingReasonCode::PolicyDenied,
            policy_decision_id: Some("secret_policy_decision".to_owned()),
        });

        assert_eq!(
            input.into_client_safe_event(),
            Err(GatewayRelayMappingError::SensitivePattern {
                field: "policy_decision_id"
            })
        );

        let mut input = base_input(GatewayNormalizedStreamEvent::Started, 0);
        input.route = Some(GatewayRouteEvidence {
            route_type: GatewayRouteType::Denied,
            reason_code: GatewayRoutingReasonCode::PolicyDenied,
            policy_decision_id: Some("decision_gateway_stream_4".to_owned()),
        });

        assert_eq!(
            input.into_client_safe_event(),
            Err(GatewayRelayMappingError::InvalidReference {
                field: "policy_decision_id"
            })
        );
    }
}
