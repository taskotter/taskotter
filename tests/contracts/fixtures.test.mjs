import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { validateWorkflowSemantics } from "./workflowSemanticValidation.mjs";

const root = process.cwd();

async function readJson(relativePath) {
  const source = await readFile(path.join(root, relativePath), "utf8");
  return JSON.parse(source);
}

function validate(
  schema,
  value,
  location = schema.title ?? "value",
  root = schema,
) {
  if (schema.const !== undefined) {
    assert.equal(value, schema.const, `${location} const mismatch`);
  }
  if (schema.enum) {
    assert.ok(
      schema.enum.includes(value),
      `${location} must be one of ${schema.enum.join(", ")}`,
    );
  }
  if (schema.type) {
    if (schema.type === "object") {
      assert.equal(typeof value, "object", `${location} must be object`);
      assert.notEqual(value, null, `${location} must not be null`);
      assert.ok(!Array.isArray(value), `${location} must not be array`);
    } else if (schema.type === "array") {
      assert.ok(Array.isArray(value), `${location} must be array`);
    } else if (schema.type === "integer") {
      assert.equal(
        Number.isInteger(value),
        true,
        `${location} must be integer`,
      );
    } else if (schema.type === "number") {
      assert.equal(typeof value, "number", `${location} must be number`);
    } else if (schema.type === "null") {
      assert.equal(value, null, `${location} must be null`);
    } else {
      assert.equal(
        typeof value,
        schema.type,
        `${location} must be ${schema.type}`,
      );
    }
  }
  if (schema.not && schemaMatches(root, schema.not, value)) {
    throw new Error(`${location} must not match forbidden schema`);
  }
  if (schema.oneOf) {
    const matchCount = schema.oneOf.filter((childSchema) =>
      schemaMatches(root, resolveRef(root, childSchema), value),
    ).length;
    assert.equal(matchCount, 1, `${location} must match exactly one schema`);
  }
  if (schema.pattern && typeof value === "string") {
    assert.match(
      value,
      new RegExp(schema.pattern),
      `${location} pattern mismatch`,
    );
  }
  if (schema.minLength !== undefined && typeof value === "string") {
    assert.ok(
      value.length >= schema.minLength,
      `${location} must have minLength ${schema.minLength}`,
    );
  }
  if (schema.minimum !== undefined && typeof value === "number") {
    assert.ok(
      value >= schema.minimum,
      `${location} must be >= ${schema.minimum}`,
    );
  }
  if (schema.maximum !== undefined && typeof value === "number") {
    assert.ok(
      value <= schema.maximum,
      `${location} must be <= ${schema.maximum}`,
    );
  }
  if (schema.minItems !== undefined && Array.isArray(value)) {
    assert.ok(
      value.length >= schema.minItems,
      `${location} must have at least ${schema.minItems} items`,
    );
  }
  if (schema.required) {
    for (const key of schema.required) {
      assert.ok(Object.hasOwn(value, key), `${location}.${key} is required`);
    }
  }
  if (
    schema.additionalProperties === false &&
    schema.properties &&
    value &&
    typeof value === "object" &&
    !Array.isArray(value)
  ) {
    for (const key of Object.keys(value)) {
      assert.ok(
        Object.hasOwn(schema.properties, key),
        `${location}.${key} is not allowed`,
      );
    }
  }
  if (
    schema.propertyNames &&
    value &&
    typeof value === "object" &&
    !Array.isArray(value)
  ) {
    for (const key of Object.keys(value)) {
      validate(
        resolveRef(root, schema.propertyNames),
        key,
        `${location}.${key} property name`,
        root,
      );
    }
  }
  if (
    schema.additionalProperties &&
    schema.additionalProperties !== false &&
    value &&
    typeof value === "object" &&
    !Array.isArray(value)
  ) {
    for (const [key, childValue] of Object.entries(value)) {
      if (!schema.properties || !Object.hasOwn(schema.properties, key)) {
        validate(
          resolveRef(root, schema.additionalProperties),
          childValue,
          `${location}.${key}`,
          root,
        );
      }
    }
  }
  if (
    schema.properties &&
    value &&
    typeof value === "object" &&
    !Array.isArray(value)
  ) {
    for (const [key, childSchema] of Object.entries(schema.properties)) {
      if (Object.hasOwn(value, key))
        validate(
          resolveRef(root, childSchema),
          value[key],
          `${location}.${key}`,
          root,
        );
    }
  }
  if (schema.items && Array.isArray(value)) {
    value.forEach((item, index) =>
      validate(
        resolveRef(root, schema.items),
        item,
        `${location}[${index}]`,
        root,
      ),
    );
  }
  if (schema.allOf) {
    for (const childSchema of schema.allOf) {
      if (childSchema.if && schemaMatches(root, childSchema.if, value)) {
        validate(childSchema.then, value, location, root);
      }
    }
  }
}

function resolveRef(rootSchema, schema) {
  if (!schema.$ref) return schema;
  if (!schema.$ref.startsWith("#/$defs/")) {
    throw new Error(`Unsupported test ref: ${schema.$ref}`);
  }
  const name = schema.$ref.slice("#/$defs/".length);
  return rootSchema.$defs[name];
}

function schemaMatches(rootSchema, schema, value) {
  try {
    validate(resolveRef(rootSchema, schema), value, "conditional", rootSchema);
    return true;
  } catch {
    return false;
  }
}

test("runtime fixtures match their canonical schemas", async () => {
  const cases = [
    [
      "contracts/schemas/policy-decision.schema.json",
      "contracts/fixtures/policy-decision.allow.runner-job.json",
    ],
    [
      "contracts/schemas/usage-event.schema.json",
      "contracts/fixtures/usage-event.gateway-request.json",
    ],
    [
      "contracts/schemas/audit-event.schema.json",
      "contracts/fixtures/audit-event.policy-denied.json",
    ],
    [
      "contracts/schemas/policy-decision.schema.json",
      "contracts/fixtures/policy-decision.deny.high-risk-runtime.json",
    ],
    [
      "contracts/schemas/usage-event.schema.json",
      "contracts/fixtures/usage-event.high-risk-runtime-denied.json",
    ],
    [
      "contracts/schemas/audit-event.schema.json",
      "contracts/fixtures/audit-event.high-risk-runtime-denied.json",
    ],
    [
      "contracts/schemas/workflow-definition.schema.json",
      "contracts/fixtures/workflow-definition.automation-contract.json",
    ],
  ];

  for (const [schemaPath, fixturePath] of cases) {
    validate(await readJson(schemaPath), await readJson(fixturePath));
  }
});

test("high-risk runtime fixtures stay deny-by-default and metered by capability", async () => {
  const decision = await readJson(
    "contracts/fixtures/policy-decision.deny.high-risk-runtime.json",
  );
  const usage = await readJson(
    "contracts/fixtures/usage-event.high-risk-runtime-denied.json",
  );
  const audit = await readJson(
    "contracts/fixtures/audit-event.high-risk-runtime-denied.json",
  );
  const [gate] = decision.constraints.high_risk_capabilities;

  assert.equal(decision.effect, "deny");
  assert.equal(gate.enabled, false);
  assert.equal(gate.effect, "deny");
  assert.equal(gate.capability, "gateway.hosted_mcp_billing");
  assert.equal(gate.feature_flag, "gateway.hosted_mcp_billing.enabled");
  assert.equal(usage.payload.measurements.runtime_capability, gate.capability);
  assert.equal(usage.payload.measurements.metering_unit, gate.metering_unit);
  assert.equal(audit.payload.runtime_capability, gate.capability);
  assert.equal(audit.payload.feature_flag, gate.feature_flag);
  assert.equal(audit.policy_decision_id, decision.decision_id);
});

test("workflow contract captures automation safety requirements", async () => {
  const workflow = await readJson(
    "contracts/fixtures/workflow-definition.automation-contract.json",
  );
  const triggerTypes = new Set(
    workflow.triggers.map((trigger) => trigger.type),
  );

  for (const triggerType of ["cron", "webhook", "internal_event"]) {
    assert.ok(
      triggerTypes.has(triggerType),
      `${triggerType} trigger is required`,
    );
  }

  const webhook = workflow.triggers.find(
    (trigger) => trigger.type === "webhook",
  ).webhook;
  assert.equal(webhook.signature_verification.required, true);
  assert.ok(
    webhook.signature_verification.secret_ref.secret_ref.startsWith("sec_"),
  );
  assert.equal(webhook.replay_protection.required, true);
  assert.ok(webhook.replay_protection.window_seconds >= 60);

  const protectedSteps = workflow.jobs.flatMap((job) =>
    job.steps.filter((step) => step.side_effect.classification === "protected"),
  );
  assert.ok(protectedSteps.length > 0, "fixture must include a protected step");
  for (const step of protectedSteps) {
    assert.ok(
      step.approval_gate_ref,
      `${step.id} requires approval before side effect`,
    );
    assert.ok(
      workflow.approval_gates.some(
        (gate) => gate.id === step.approval_gate_ref,
      ),
      `${step.id} approval gate must exist`,
    );
  }
  validateWorkflowSemantics(workflow);
});

test("workflow fixtures only carry secret and integration references", async () => {
  const workflow = await readJson(
    "contracts/fixtures/workflow-definition.automation-contract.json",
  );
  const serialized = JSON.stringify(workflow);
  const forbiddenSecretShapes = [
    /api[_-]?key/i,
    /access[_-]?token/i,
    /refresh[_-]?token/i,
    /private[_-]?key/i,
    /client[_-]?secret/i,
    /bearer\s+[a-z0-9._-]+/i,
  ];

  for (const pattern of forbiddenSecretShapes) {
    assert.doesNotMatch(serialized, pattern);
  }
  assert.match(serialized, /"secret_ref":"sec_/);
  assert.match(serialized, /"integration_ref":"int_/);
});

test("workflow schema rejects raw credential-shaped step inputs", async () => {
  const schema = await readJson(
    "contracts/schemas/workflow-definition.schema.json",
  );
  const invalidWorkflow = await readJson(
    "contracts/fixtures/workflow-definition.raw-credential-input.invalid.json",
  );

  assert.throws(
    () => validate(schema, invalidWorkflow),
    /access_token property name/,
  );
});

test("workflow semantic validation requires protected approval gates", async () => {
  const schema = await readJson(
    "contracts/schemas/workflow-definition.schema.json",
  );
  const cases = [
    [
      "contracts/fixtures/workflow-definition.missing-approval-gate.invalid.json",
      /approval_gate_ref must reference an approval gate/,
    ],
    [
      "contracts/fixtures/workflow-definition.wrong-approval-gate.invalid.json",
      /approval gate must be required before protected_side_effect/,
    ],
  ];

  for (const [fixturePath, expectedError] of cases) {
    const workflow = await readJson(fixturePath);
    validate(schema, workflow);
    assert.throws(() => validateWorkflowSemantics(workflow), expectedError);
  }
});

test("generated schema artifacts mirror canonical schema sources", async () => {
  const schemaFileNames = (await readdir(path.join(root, "contracts/schemas")))
    .filter((fileName) => fileName.endsWith(".schema.json"))
    .sort();

  for (const fileName of schemaFileNames) {
    assert.deepEqual(
      await readJson(`packages/schemas/json/${fileName}`),
      await readJson(`contracts/schemas/${fileName}`),
      `${fileName} generated artifact must mirror canonical source`,
    );
  }
});

test("OpenAPI exposes the first control-plane resource surface", async () => {
  const openapi = await readJson(
    "contracts/openapi/taskotter-control-plane.openapi.json",
  );
  const expectedPaths = [
    "/users",
    "/working-groups",
    "/issues",
    "/issues/{issue_id}/comments",
    "/agents",
    "/skills",
    "/integrations",
    "/providers",
    "/workflows",
    "/usage/events",
    "/audit/events",
  ];

  for (const route of expectedPaths) {
    assert.ok(openapi.paths[route], `${route} must be present`);
  }
  assert.ok(
    openapi.paths["/usage/events"].get,
    "usage read surface must be present",
  );
  assert.ok(
    openapi.paths["/usage/events"].post,
    "usage ingest surface must be present",
  );
  assert.equal(openapi.info.version, "0.1.0");
});

test("runtime events use the canonical event envelope and required correlation chain", async () => {
  const usage = await readJson(
    "contracts/fixtures/usage-event.gateway-request.json",
  );
  const audit = await readJson(
    "contracts/fixtures/audit-event.policy-denied.json",
  );

  for (const event of [usage, audit]) {
    assert.match(event.id, /^evt_/);
    assert.equal(typeof event.type, "string");
    assert.equal(event.version, "0.1.0");
    assert.match(event.correlation_id, /^corr_/);
    assert.match(event.request_id, /^req_/);
    assert.match(event.policy_decision_id, /^poldec_/);
    assert.equal(typeof event.payload, "object");
    assert.ok(
      !Object.hasOwn(event, "event_id"),
      "event_id must not replace canonical id",
    );
    assert.ok(
      !Object.hasOwn(event, "schema_version"),
      "event envelopes use version, not schema_version",
    );
  }
});

test("policy decisions require provenance and audit correlation fields", async () => {
  const schema = await readJson(
    "contracts/schemas/policy-decision.schema.json",
  );
  const required = new Set(schema.required);

  for (const field of [
    "policy_version",
    "policy_snapshot_id",
    "reason_code",
    "correlation_id",
    "request_id",
    "provenance",
  ]) {
    assert.ok(required.has(field), `policy decision must require ${field}`);
  }
});

test("comment writes derive Working Group scope from the issue path", async () => {
  const openapi = await readJson(
    "contracts/openapi/taskotter-control-plane.openapi.json",
  );
  const requestSchema = openapi.components.schemas.CreateCommentRequest;
  const post = openapi.paths["/issues/{issue_id}/comments"].post;

  assert.deepEqual(requestSchema.required, ["body"]);
  assert.ok(!Object.hasOwn(requestSchema.properties, "working_group_id"));
  assert.match(
    post.description,
    /derives the Working Group from the issue resource/,
  );
  assert.ok(
    post.responses["409"],
    "WG mismatch rejection response must be documented",
  );
});

test("error details are restricted to a safe allowlist", async () => {
  const openapi = await readJson(
    "contracts/openapi/taskotter-control-plane.openapi.json",
  );
  const details =
    openapi.components.schemas.ErrorEnvelope.properties.error.properties
      .details;
  const allowedKeys = Object.keys(details.items.properties).sort();

  assert.equal(details.items.additionalProperties, false);
  assert.deepEqual(allowedKeys, ["code", "field", "message", "redacted"]);
  assert.match(
    details.description,
    /Do not include request bodies, credentials/,
  );
});
