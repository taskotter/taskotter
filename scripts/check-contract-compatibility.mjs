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
