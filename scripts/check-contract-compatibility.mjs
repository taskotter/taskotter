import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();

async function readJson(relativePath) {
  const source = await readFile(path.join(root, relativePath), "utf8");
  return JSON.parse(source);
}

function assertSupportedVersion(matrix, surface, field, version, location) {
  const supported = matrix[surface][field];
  assert.ok(Array.isArray(supported), `${surface}.${field} must be an array`);
  assert.ok(
    supported.includes(version),
    `${location} version ${version} must be declared in ${surface}.${field}`,
  );
}

const matrix = await readJson("contracts/compatibility-matrix.json");
const openapi = await readJson(
  "contracts/openapi/taskotter-control-plane.openapi.json",
);
const policyDecision = await readJson(
  "contracts/fixtures/policy-decision.allow.runner-job.json",
);
const usageEvent = await readJson(
  "contracts/fixtures/usage-event.gateway-request.json",
);
const auditEvent = await readJson(
  "contracts/fixtures/audit-event.policy-denied.json",
);
const remoteDispatch = await readJson(
  "contracts/compatibility-fixtures/remote/job-dispatch-echo.json",
);
const gatewayRequest = await readJson(
  "contracts/compatibility-fixtures/gateway/scoped-model-request.json",
);
const syntheticChain = await readJson(
  "contracts/fixtures/audit-chain.synthetic-correlation-run.json",
);

assert.equal(
  openapi.info.version,
  matrix.control_plane.openapi_version,
  "OpenAPI version must match the control-plane compatibility declaration",
);
assertSupportedVersion(
  matrix,
  "control_plane",
  "policy_decision_versions",
  policyDecision.schema_version,
  "policy decision fixture",
);

for (const [name, event] of [
  ["usage event fixture", usageEvent],
  ["audit event fixture", auditEvent],
]) {
  assertSupportedVersion(
    matrix,
    "control_plane",
    "event_envelope_versions",
    event.version,
    name,
  );
  assert.match(event.id, /^evt_/, `${name} must use canonical event id`);
  assert.match(
    event.correlation_id,
    /^corr_/,
    `${name} must preserve correlation chain`,
  );
  assert.match(
    event.request_id,
    /^req_/,
    `${name} must preserve request chain`,
  );
}

assert.equal(
  remoteDispatch.protocol_version,
  matrix.remote.fixture_protocol,
  "remote dispatch compatibility fixture must use the declared protocol",
);
assertSupportedVersion(
  matrix,
  "remote",
  "supported_protocol_versions",
  remoteDispatch.protocol_version,
  "remote dispatch fixture",
);
assert.equal(remoteDispatch.message_type, "job.dispatch");
assert.match(remoteDispatch.payload.policy.policy_version, /^policy_/);

assert.equal(
  gatewayRequest.protocol_version,
  matrix.gateway.fixture_protocol,
  "gateway request compatibility fixture must use the declared protocol",
);
assertSupportedVersion(
  matrix,
  "gateway",
  "supported_protocol_versions",
  gatewayRequest.protocol_version,
  "gateway scoped model request fixture",
);
assert.equal(gatewayRequest.policy.kind, "decision_ref");
assert.match(gatewayRequest.policy.decision_ref, /^poldec_/);
assert.ok(
  gatewayRequest.credential_ref.reference.startsWith("secret_ref_"),
  "gateway compatibility fixture must use scoped secret references only",
);

assert.equal(
  syntheticChain.cross_repo_evidence.remote.protocol_version,
  matrix.remote.fixture_protocol,
  "synthetic chain must reference the declared remote fixture protocol",
);
assert.equal(
  syntheticChain.cross_repo_evidence.remote.message_type,
  remoteDispatch.message_type,
  "synthetic chain must preserve the remote dispatch message type",
);
assert.equal(
  syntheticChain.cross_repo_evidence.gateway.protocol_version,
  matrix.gateway.fixture_protocol,
  "synthetic chain must reference the declared gateway fixture protocol",
);

const chainEvents = syntheticChain.events ?? [];
const requiredStages = new Set([
  "user_request",
  "policy_decision",
  "plan_approval_requested",
  "plan_approved",
  "approval_requested",
  "runner_dispatch",
  "gateway_request",
  "mcp_call_denied",
  "usage_event",
  "audit_event",
  "artifact_log_event",
  "evidence_imported",
  "review_packet_generated",
  "human_decision_done",
  "evidence_missing",
  "human_decision_rework",
  "final_result",
]);
for (const stage of requiredStages) {
  assert.ok(
    chainEvents.some((event) => event.stage === stage),
    `synthetic chain must include ${stage}`,
  );
}
for (const event of chainEvents) {
  assert.equal(
    event.correlation_id,
    syntheticChain.chain.correlation_id,
    `${event.stage} must preserve the chain correlation id`,
  );
  assert.equal(
    event.request_id,
    syntheticChain.chain.request_id,
    `${event.stage} must preserve the chain request id`,
  );
  assert.ok(
    ["internal_reference_only", "redacted_summary"].includes(event.redaction),
    `${event.stage} must expose only redacted or internal-reference evidence`,
  );
  assert.ok(
    Object.hasOwn(event.payload, "workflow_path"),
    `${event.stage} must include workflow path discriminator`,
  );
}

const prototypePaths = new Map(
  (syntheticChain.prototype_paths ?? []).map((path) => [path.path, path]),
);
assert.deepEqual([...prototypePaths.keys()].sort(), [
  "denied",
  "done_approved",
  "missing_evidence",
  "rework_requested",
]);
assert.equal(prototypePaths.get("done_approved").terminal_state, "done");
assert.equal(prototypePaths.get("rework_requested").terminal_state, "rework");
assert.equal(prototypePaths.get("missing_evidence").terminal_state, "rework");
assert.equal(prototypePaths.get("denied").terminal_state, "denied");

const eventPathKey = (stage, workflowPath) => `${stage}:${workflowPath}`;
const eventsByStageAndPath = new Map(
  chainEvents.map((event) => [
    eventPathKey(event.stage, event.payload.workflow_path),
    event,
  ]),
);
for (const path of prototypePaths.values()) {
  for (const stage of path.required_stages) {
    assert.ok(
      eventsByStageAndPath.has(eventPathKey(stage, path.path)),
      `${path.path} path must reconstruct ${stage} using its own workflow_path`,
    );
  }
}

const donePlanApproval = eventsByStageAndPath.get(
  eventPathKey("plan_approval_requested", "done_approved"),
);
const donePlanApproved = eventsByStageAndPath.get(
  eventPathKey("plan_approved", "done_approved"),
);
const doneEvidenceImport = eventsByStageAndPath.get(
  eventPathKey("evidence_imported", "done_approved"),
);
const doneReviewPacket = eventsByStageAndPath.get(
  eventPathKey("review_packet_generated", "done_approved"),
);
const doneDecision = eventsByStageAndPath.get(
  eventPathKey("human_decision_done", "done_approved"),
);
const reworkEvidenceImport = eventsByStageAndPath.get(
  eventPathKey("evidence_imported", "rework_requested"),
);
const reworkReviewPacket = eventsByStageAndPath.get(
  eventPathKey("review_packet_generated", "rework_requested"),
);
const reworkFinalResult = eventsByStageAndPath.get(
  eventPathKey("final_result", "rework_requested"),
);
const missingEvidence = eventsByStageAndPath.get(
  eventPathKey("evidence_missing", "missing_evidence"),
);
const missingReviewPacket = eventsByStageAndPath.get(
  eventPathKey("review_packet_generated", "missing_evidence"),
);
const missingFinalResult = eventsByStageAndPath.get(
  eventPathKey("final_result", "missing_evidence"),
);

assert.equal(donePlanApproval.approval_id, syntheticChain.chain.approval_id);
assert.equal(donePlanApproved.payload.decision, "approved");
assert.equal(
  doneEvidenceImport.payload.evidence_import_id,
  syntheticChain.chain.evidence_import_id,
);
assert.equal(
  doneReviewPacket.payload.review_packet_id,
  syntheticChain.chain.review_packet_id,
);
assert.equal(doneDecision.payload.decision, "done");
assert.equal(
  eventsByStageAndPath.get(
    eventPathKey("human_decision_rework", "rework_requested"),
  ).payload.decision,
  "rework",
);
assert.equal(missingEvidence.payload.decision_hint, "rework");
assert.equal(reworkFinalResult.payload.status, "rework");
assert.equal(missingFinalResult.payload.status, "rework");
assert.notEqual(
  reworkEvidenceImport.payload.evidence_import_id,
  doneEvidenceImport.payload.evidence_import_id,
  "rework evidence import must not reuse done path evidence",
);
assert.notEqual(
  reworkReviewPacket.payload.review_packet_id,
  doneReviewPacket.payload.review_packet_id,
  "rework review packet must not reuse done path packet",
);
assert.notEqual(
  missingReviewPacket.payload.review_packet_id,
  doneReviewPacket.payload.review_packet_id,
  "missing evidence review packet must not reuse done path packet",
);

const negativeCases = new Map(
  (syntheticChain.negative_cases ?? []).map((negativeCase) => [
    negativeCase.case,
    negativeCase,
  ]),
);
assert.deepEqual([...negativeCases.keys()].sort(), [
  "malformed_correlation_id",
  "missing_correlation_id",
]);
for (const negativeCase of negativeCases.values()) {
  assert.ok(
    chainEvents.some((event) => event.stage === negativeCase.target_stage),
    `${negativeCase.case} must target an existing stage`,
  );
  assert.equal(negativeCase.field, "correlation_id");
  assert.equal(negativeCase.expected_error, "correlation_id");
  if (negativeCase.operation === "set_field") {
    assert.doesNotMatch(
      negativeCase.value,
      /^corr_[0-9A-HJKMNP-TV-Z]{26}$/,
      `${negativeCase.case} must carry a malformed correlation value`,
    );
  }
}

for (const event of chainEvents) {
  assert.ok(
    !Object.hasOwn(event, "event_id"),
    `${event.stage} must use canonical id, not event_id`,
  );
  for (const field of [
    "id",
    "type",
    "version",
    "occurred_at",
    "source",
    "working_group_id",
    "actor",
    "resource",
    "payload",
  ]) {
    assert.ok(Object.hasOwn(event, field), `${event.stage} missing ${field}`);
  }
}

const canonicalReferences = new Map(
  chainEvents
    .filter((event) => event.event_shape === "canonical_event_reference")
    .map((event) => [event.stage, event]),
);
assert.deepEqual([...canonicalReferences.keys()].sort(), [
  "audit_event",
  "usage_event",
]);
assert.equal(
  canonicalReferences.get("usage_event").canonical_fixture_path,
  "contracts/fixtures/usage-event.high-risk-runtime-denied.json",
  "usage chain stage must reference the canonical denied usage fixture",
);
assert.equal(
  canonicalReferences.get("audit_event").canonical_fixture_path,
  "contracts/fixtures/audit-event.high-risk-runtime-denied.json",
  "audit chain stage must reference the canonical denied audit fixture",
);

const runnerDispatch = chainEvents.find(
  (event) => event.stage === "runner_dispatch",
);
assert.equal(
  runnerDispatch.protocol_version,
  matrix.remote.fixture_protocol,
  "runner dispatch event must stay compatible with remote protocol fixtures",
);
assert.equal(
  syntheticChain.cross_repo_evidence.remote.lineage_model,
  "dispatch_fragment_requires_control_plane_join",
  "remote dispatch cannot claim standalone full-chain reconstruction",
);
assert.equal(
  remoteDispatch.payload.correlation_id,
  "corr_fixture_echo_001",
  "remote compatibility fixture preserves existing remote fixture vocabulary",
);
assert.notEqual(
  remoteDispatch.payload.correlation_id,
  syntheticChain.chain.correlation_id,
  "remote dispatch fixture must not be mistaken for a complete synthetic chain record",
);
assert.equal(
  Object.hasOwn(remoteDispatch.payload, "request_id"),
  false,
  "remote dispatch fixture does not carry request_id; control-plane join is required",
);
assert.equal(
  Object.hasOwn(remoteDispatch.payload, "policy_decision_id"),
  false,
  "remote dispatch fixture does not carry policy_decision_id; control-plane join is required",
);
assert.ok(
  syntheticChain.cross_repo_evidence.remote.required_control_plane_join_keys.includes(
    "policy_decision_id",
  ),
  "remote lineage evidence must declare the control-plane policy decision join",
);

for (const gatewayStage of ["gateway_request", "mcp_call_denied"]) {
  const gatewayEvent = chainEvents.find(
    (event) => event.stage === gatewayStage,
  );
  assert.equal(
    gatewayEvent.protocol_version,
    matrix.gateway.fixture_protocol,
    `${gatewayStage} must stay compatible with gateway protocol fixtures`,
  );
  assert.equal(
    gatewayEvent.policy_decision_id,
    gatewayRequest.policy.decision_ref,
    `${gatewayStage} must share the gateway decision ref lineage`,
  );
}

const serializedChain = JSON.stringify(syntheticChain).toLowerCase();
for (const prohibited of [
  "api_key",
  "access_token",
  "refresh_token",
  "private_key",
  "client_secret",
  "bearer ",
  "password",
  "raw_prompt",
  "raw_log",
  "artifact_body",
  "transcript_copy",
  "-----begin",
]) {
  assert.ok(
    !serializedChain.includes(prohibited),
    `synthetic chain must not include sensitive marker ${prohibited}`,
  );
}
