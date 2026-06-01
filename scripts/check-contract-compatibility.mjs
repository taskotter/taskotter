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
  "approval_requested",
  "runner_dispatch",
  "gateway_request",
  "mcp_call_denied",
  "usage_event",
  "audit_event",
  "artifact_log_event",
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
  "-----begin",
]) {
  assert.ok(
    !serializedChain.includes(prohibited),
    `synthetic chain must not include sensitive marker ${prohibited}`,
  );
}
