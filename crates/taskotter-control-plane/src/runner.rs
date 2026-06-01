use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerControlPlaneFlags {
    #[serde(default)]
    pub runner_dispatch_enabled: bool,
    #[serde(default)]
    pub https_fallback_enabled: bool,
    #[serde(default)]
    pub private_network_egress_enabled: bool,
    #[serde(default)]
    pub local_tools_enabled: bool,
    #[serde(default)]
    pub local_llm_enabled: bool,
    #[serde(default)]
    pub computer_use_enabled: bool,
    #[serde(default)]
    pub external_agent_adapters_enabled: bool,
    #[serde(default = "default_kill_switch_engaged")]
    pub kill_switch_engaged: bool,
}

impl Default for RunnerControlPlaneFlags {
    fn default() -> Self {
        Self {
            runner_dispatch_enabled: false,
            https_fallback_enabled: false,
            private_network_egress_enabled: false,
            local_tools_enabled: false,
            local_llm_enabled: false,
            computer_use_enabled: false,
            external_agent_adapters_enabled: false,
            kill_switch_engaged: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunnerCapability {
    HttpsFallback,
    PrivateNetworkEgress,
    LocalTools,
    LocalLlm,
    ComputerUse,
    ExternalAgentAdapters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerDispatchRequest {
    pub working_group_id: String,
    pub run_id: String,
    pub runner_id: String,
    #[serde(default)]
    pub required_capabilities: Vec<RunnerCapability>,
    #[serde(default, rename = "control_plane_flags", skip_serializing)]
    #[schema(ignore)]
    pub ignored_client_control_plane_flags: Option<serde_json::Value>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunnerDispatchStatus {
    Queued,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerDispatchDiagnostic {
    pub reason_code: String,
    pub message_key: String,
    pub audit_event_type: String,
    pub safe_summary: String,
    #[serde(default)]
    pub capability: Option<RunnerCapability>,
    #[serde(default)]
    pub feature_flag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RunnerDispatchDecision {
    pub accepted: bool,
    pub status: RunnerDispatchStatus,
    pub run_id: String,
    pub runner_id: String,
    pub working_group_id: String,
    pub correlation_id: String,
    pub request_id: String,
    pub evaluated_at: String,
    pub diagnostic: RunnerDispatchDiagnostic,
}

impl RunnerDispatchRequest {
    pub fn evaluate_for_dispatch(
        &self,
        control_plane_flags: &RunnerControlPlaneFlags,
    ) -> RunnerDispatchDecision {
        let diagnostic = if control_plane_flags.kill_switch_engaged {
            RunnerDispatchDiagnostic {
                reason_code: "control_plane_kill_switch_engaged".to_owned(),
                message_key: "runner.dispatch.refused.kill_switch".to_owned(),
                audit_event_type: "runner.dispatch.refused".to_owned(),
                safe_summary: "Runner dispatch refused by control-plane kill switch.".to_owned(),
                capability: None,
                feature_flag: Some("runner.kill_switch.engaged".to_owned()),
            }
        } else if !control_plane_flags.runner_dispatch_enabled {
            RunnerDispatchDiagnostic {
                reason_code: "runner_dispatch_disabled".to_owned(),
                message_key: "runner.dispatch.refused.disabled".to_owned(),
                audit_event_type: "runner.dispatch.refused".to_owned(),
                safe_summary:
                    "Runner dispatch refused because the control-plane dispatch flag is disabled."
                        .to_owned(),
                capability: None,
                feature_flag: Some("runner.dispatch.enabled".to_owned()),
            }
        } else if let Some(capability) = self
            .required_capabilities
            .iter()
            .find(|capability| !control_plane_flags.capability_enabled(capability))
        {
            RunnerDispatchDiagnostic {
                reason_code: "runner_capability_disabled".to_owned(),
                message_key: "runner.dispatch.refused.capability_disabled".to_owned(),
                audit_event_type: "runner.dispatch.refused".to_owned(),
                safe_summary: "Runner dispatch refused before execution because a required capability is disabled."
                    .to_owned(),
                capability: Some(capability.clone()),
                feature_flag: Some(capability.feature_flag().to_owned()),
            }
        } else {
            RunnerDispatchDiagnostic {
                reason_code: "runner_dispatch_allowed".to_owned(),
                message_key: "runner.dispatch.queued".to_owned(),
                audit_event_type: "runner.dispatch.queued".to_owned(),
                safe_summary: "Runner dispatch accepted by current control-plane feature flags."
                    .to_owned(),
                capability: None,
                feature_flag: Some("runner.dispatch.enabled".to_owned()),
            }
        };

        let accepted = diagnostic.reason_code == "runner_dispatch_allowed";

        RunnerDispatchDecision {
            accepted,
            status: if accepted {
                RunnerDispatchStatus::Queued
            } else {
                RunnerDispatchStatus::Refused
            },
            run_id: self.run_id.clone(),
            runner_id: self.runner_id.clone(),
            working_group_id: self.working_group_id.clone(),
            correlation_id: self
                .correlation_id
                .clone()
                .unwrap_or_else(|| "corr_runner_dispatch_local".to_owned()),
            request_id: self
                .request_id
                .clone()
                .unwrap_or_else(|| "req_runner_dispatch_local".to_owned()),
            evaluated_at: "2026-01-01T00:00:00.000Z".to_owned(),
            diagnostic,
        }
    }
}

impl RunnerControlPlaneFlags {
    fn capability_enabled(&self, capability: &RunnerCapability) -> bool {
        match capability {
            RunnerCapability::HttpsFallback => self.https_fallback_enabled,
            RunnerCapability::PrivateNetworkEgress => self.private_network_egress_enabled,
            RunnerCapability::LocalTools => self.local_tools_enabled,
            RunnerCapability::LocalLlm => self.local_llm_enabled,
            RunnerCapability::ComputerUse => self.computer_use_enabled,
            RunnerCapability::ExternalAgentAdapters => self.external_agent_adapters_enabled,
        }
    }
}

impl RunnerCapability {
    fn feature_flag(&self) -> &'static str {
        match self {
            RunnerCapability::HttpsFallback => "runner.https_fallback.enabled",
            RunnerCapability::PrivateNetworkEgress => "runner.private_network_egress.enabled",
            RunnerCapability::LocalTools => "runner.local_tools.enabled",
            RunnerCapability::LocalLlm => "runner.local_llm.enabled",
            RunnerCapability::ComputerUse => "runner.computer_use.enabled",
            RunnerCapability::ExternalAgentAdapters => "runner.external_agent_adapters.enabled",
        }
    }
}

fn default_kill_switch_engaged() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch_request() -> RunnerDispatchRequest {
        RunnerDispatchRequest {
            working_group_id: "wg_1".to_owned(),
            run_id: "run_1".to_owned(),
            runner_id: "runner_1".to_owned(),
            required_capabilities: Vec::new(),
            ignored_client_control_plane_flags: None,
            correlation_id: Some("corr_1".to_owned()),
            request_id: Some("req_1".to_owned()),
        }
    }

    #[test]
    fn default_flags_refuse_dispatch_with_kill_switch_reason() {
        let request = dispatch_request();
        let decision = request.evaluate_for_dispatch(&RunnerControlPlaneFlags::default());

        assert!(!decision.accepted);
        assert_eq!(decision.status, RunnerDispatchStatus::Refused);
        assert_eq!(
            decision.diagnostic.reason_code,
            "control_plane_kill_switch_engaged"
        );
        assert_eq!(
            decision.diagnostic.audit_event_type,
            "runner.dispatch.refused"
        );
        assert_eq!(
            decision.diagnostic.feature_flag,
            Some("runner.kill_switch.engaged".to_owned())
        );
    }

    #[test]
    fn disabled_capability_refuses_before_dispatch_with_diagnostic_reason() {
        let mut request = dispatch_request();
        request.required_capabilities = vec![RunnerCapability::LocalTools];
        let control_plane_flags = RunnerControlPlaneFlags {
            kill_switch_engaged: false,
            runner_dispatch_enabled: true,
            ..RunnerControlPlaneFlags::default()
        };

        let decision = request.evaluate_for_dispatch(&control_plane_flags);

        assert!(!decision.accepted);
        assert_eq!(decision.status, RunnerDispatchStatus::Refused);
        assert_eq!(
            decision.diagnostic.reason_code,
            "runner_capability_disabled"
        );
        assert_eq!(
            decision.diagnostic.capability,
            Some(RunnerCapability::LocalTools)
        );
        assert_eq!(
            decision.diagnostic.feature_flag,
            Some("runner.local_tools.enabled".to_owned())
        );
        assert_eq!(
            decision.diagnostic.message_key,
            "runner.dispatch.refused.capability_disabled"
        );
    }

    #[test]
    fn enabled_dispatch_and_capability_can_queue_runner_job() {
        let mut request = dispatch_request();
        request.required_capabilities = vec![RunnerCapability::LocalTools];
        let control_plane_flags = RunnerControlPlaneFlags {
            kill_switch_engaged: false,
            runner_dispatch_enabled: true,
            local_tools_enabled: true,
            ..RunnerControlPlaneFlags::default()
        };

        let decision = request.evaluate_for_dispatch(&control_plane_flags);

        assert!(decision.accepted);
        assert_eq!(decision.status, RunnerDispatchStatus::Queued);
        assert_eq!(decision.diagnostic.reason_code, "runner_dispatch_allowed");
    }
}
