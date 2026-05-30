use crate::domain::{PrincipalRef, ResourceId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageEvent {
    pub id: ResourceId,
    pub working_group_id: ResourceId,
    pub principal: PrincipalRef,
    pub subject: UsageSubject,
    pub units: Vec<UsageUnit>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageSubject {
    ModelCall {
        provider_id: ResourceId,
        model: String,
    },
    ToolCall {
        integration_id: ResourceId,
    },
    AutomationRun {
        workflow_id: ResourceId,
    },
    RunnerJob {
        runner_id: ResourceId,
    },
    GatewayRequest {
        gateway_id: ResourceId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageUnit {
    pub kind: UsageUnitKind,
    pub quantity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageUnitKind {
    InputToken,
    OutputToken,
    ToolInvocation,
    RuntimeMillisecond,
    StoredByte,
    EstimatedCostMicrousd,
}

impl UsageEvent {
    pub fn total_estimated_cost_microusd(&self) -> u64 {
        self.units
            .iter()
            .filter(|unit| unit.kind == UsageUnitKind::EstimatedCostMicrousd)
            .map(|unit| unit.quantity)
            .sum()
    }
}
