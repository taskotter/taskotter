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
      if (!childSchema.if) {
        validate(resolveRef(root, childSchema), value, location, root);
      } else if (schemaMatches(root, childSchema.if, value)) {
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
      "contracts/schemas/usage-event.schema.json",
      "contracts/fixtures/usage-event.runner-job-settled.json",
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
  assert.equal(usage.status, "denied");
  assert.equal(usage.denial_reason, "protected_capability_disabled");
  assert.equal(audit.payload.runtime_capability, gate.capability);
  assert.equal(audit.payload.feature_flag, gate.feature_flag);
  assert.equal(audit.policy_decision_id, decision.decision_id);
});

test("runner-sensitive snapshot reconstructs correlation chain without raw secrets", async () => {
  const usageSchema = await readJson(
    "contracts/schemas/usage-event.schema.json",
  );
  const auditSchema = await readJson(
    "contracts/schemas/audit-event.schema.json",
  );
  const snapshot = await readJson(
    "contracts/fixtures/runner-sensitive-correlation-snapshot.json",
  );
  const allEvents = [
    ...snapshot.timeline_events,
    ...snapshot.usage_events,
    ...snapshot.audit_events,
    ...snapshot.operations_audit_events,
    ...snapshot.health_events,
  ];
  const eventsById = new Map(allEvents.map((event) => [event.id, event]));
  const [reconstruction] = snapshot.correlation_reconstruction;

  assert.deepEqual(Object.keys(snapshot.scenario_coverage).sort(), [
    "completion_failure",
    "credential_use",
    "denied_egress",
    "dispatch_accepted",
    "dispatch_rejected",
    "quarantine_event",
    "replay_rejected",
  ]);
  for (const eventId of Object.values(snapshot.scenario_coverage)) {
    assert.ok(eventsById.has(eventId), `${eventId} must exist in snapshot`);
  }

  assert.deepEqual(reconstruction.flow, [
    "request",
    "policy",
    "dispatch",
    "runner_result",
  ]);
  for (const eventId of reconstruction.ordered_event_ids) {
    const event = eventsById.get(eventId);
    assert.ok(event, `${eventId} must be reconstructable`);
    const envelope = event.envelope ?? event;
    assert.equal(envelope.correlation_id, reconstruction.correlation_id);
    assert.equal(envelope.request_id, reconstruction.request_id);
  }

  for (const usageEvent of snapshot.usage_events) {
    validate(usageSchema, usageEvent);
  }
  for (const auditEvent of snapshot.audit_events) {
    validate(auditSchema, auditEvent);
  }

  const { redaction_assertions: redactionAssertions, ...snapshotBody } =
    snapshot;
  const serialized = JSON.stringify(snapshotBody).toLowerCase();
  for (const forbidden of redactionAssertions.forbidden_substrings) {
    assert.ok(
      !serialized.includes(forbidden.toLowerCase()),
      `snapshot must not expose ${forbidden}`,
    );
  }
  assert.equal(snapshot.redaction_policy, "safe_refs_only");
});

test("usage event schema captures settlement and safe-reference contract", async () => {
  const schema = await readJson("contracts/schemas/usage-event.schema.json");
  const runnerUsage = await readJson(
    "contracts/fixtures/usage-event.runner-job-settled.json",
  );

  validate(schema, runnerUsage);
  assert.equal(runnerUsage.payload.subject.type, "runner_job");
  assert.equal(runnerUsage.status, "succeeded");
  assert.match(runnerUsage.reservation_id, /^resv_/);
  assert.equal(runnerUsage.payload.measurements.actual_cost_micros, 3900);

  const properties = schema.properties;
  assert.ok(properties.reservation_id, "reservation_id must be in schema");
  assert.ok(properties.status, "status must be in schema");
  assert.ok(properties.denial_reason, "denial_reason must be in schema");
  assert.ok(
    schema.$defs.usageSubjectRef.properties.type.enum.includes("runner_job"),
    "runner_job subject must be supported",
  );
  assert.equal(schema.$defs.safeOpaqueRef.maxLength, 80);
  assert.match(schema.$defs.idempotencyKey.pattern, /^\^usage_/);
});

test("usage event schema rejects unsafe denied and reference shapes", async () => {
  const schema = await readJson("contracts/schemas/usage-event.schema.json");
  const deniedUsage = await readJson(
    "contracts/fixtures/usage-event.high-risk-runtime-denied.json",
  );
  const unsafeReason = JSON.parse(JSON.stringify(deniedUsage));
  unsafeReason.denial_reason = "raw_prompt: bearer token";
  assert.throws(() => validate(schema, unsafeReason), /denial_reason/);

  const badIdempotency = JSON.parse(JSON.stringify(deniedUsage));
  badIdempotency.idempotency_key = "usage_access_token_body";
  assert.throws(() => validate(schema, badIdempotency), /idempotency_key/);

  const badRunnerRef = await readJson(
    "contracts/fixtures/usage-event.runner-job-settled.json",
  );
  badRunnerRef.payload.subject.id = "gwreq_01J9Z4P4BS0M9P2QJ6T8Z6W2ER";
  assert.throws(() => validate(schema, badRunnerRef), /payload.subject.id/);
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

test("synthetic audit chain reconstructs cross-surface correlation without sensitive payloads", async () => {
  const fixture = await readJson(
    "contracts/fixtures/audit-chain.synthetic-correlation-run.json",
  );
  const chainSchema = await readJson(
    "contracts/schemas/audit-chain-fixture.schema.json",
  );
  const envelopeSchema = await readJson(
    "contracts/schemas/event-envelope.schema.json",
  );
  const usageSchema = await readJson(
    "contracts/schemas/usage-event.schema.json",
  );
  const auditSchema = await readJson(
    "contracts/schemas/audit-event.schema.json",
  );
  const deniedUsage = await readJson(
    "contracts/fixtures/usage-event.high-risk-runtime-denied.json",
  );
  const deniedAudit = await readJson(
    "contracts/fixtures/audit-event.high-risk-runtime-denied.json",
  );

  validate(chainSchema, fixture);
  validate(usageSchema, deniedUsage);
  validate(auditSchema, deniedAudit);

  const expectedStages = new Set([
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
  const actualStages = new Set(fixture.events.map((event) => event.stage));
  for (const stage of expectedStages) {
    assert.ok(actualStages.has(stage), `synthetic chain must include ${stage}`);
  }

  for (const event of fixture.events) {
    assert.equal(event.correlation_id, fixture.chain.correlation_id);
    assert.equal(event.request_id, fixture.chain.request_id);
    assert.equal(event.working_group_id, fixture.chain.working_group_id);
    assert.match(event.id, /^evt_/);
    assert.ok(
      !Object.hasOwn(event, "event_id"),
      `${event.stage} must use canonical id, not event_id`,
    );
    validate(envelopeSchema, {
      id: event.id,
      type: event.type,
      version: event.version,
      occurred_at: event.occurred_at,
      source: event.source,
      working_group_id: event.working_group_id,
      actor: event.actor,
      resource: event.resource,
      correlation_id: event.correlation_id,
      request_id: event.request_id,
      ...(event.policy_decision_id
        ? { policy_decision_id: event.policy_decision_id }
        : {}),
      payload: event.payload,
    });
    assert.ok(
      ["internal_reference_only", "redacted_summary"].includes(event.redaction),
      `${event.stage} must not expose public or raw evidence`,
    );
    assert.ok(
      Object.hasOwn(event.payload, "workflow_path"),
      `${event.stage} must carry a workflow_path discriminator`,
    );
  }

  const prototypePaths = new Map(
    fixture.prototype_paths.map((path) => [path.path, path]),
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
    fixture.events.map((event) => [
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

  assert.equal(donePlanApproval.approval_id, fixture.chain.approval_id);
  assert.equal(donePlanApproved.payload.decision, "approved");
  assert.equal(
    doneEvidenceImport.payload.evidence_import_id,
    fixture.chain.evidence_import_id,
  );
  assert.equal(
    doneReviewPacket.payload.review_packet_id,
    fixture.chain.review_packet_id,
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
    fixture.negative_cases.map((negativeCase) => [
      negativeCase.case,
      negativeCase,
    ]),
  );
  assert.deepEqual([...negativeCases.keys()].sort(), [
    "malformed_correlation_id",
    "missing_correlation_id",
  ]);
  for (const negativeCase of negativeCases.values()) {
    const mutated = JSON.parse(JSON.stringify(fixture));
    const targetEvent = mutated.events.find(
      (event) => event.stage === negativeCase.target_stage,
    );
    assert.ok(targetEvent, `${negativeCase.case} target stage must exist`);
    if (negativeCase.operation === "remove_field") {
      delete targetEvent[negativeCase.field];
    } else if (negativeCase.operation === "set_field") {
      targetEvent[negativeCase.field] = negativeCase.value;
    }
    assert.throws(
      () => validate(chainSchema, mutated),
      new RegExp(negativeCase.expected_error),
      `${negativeCase.case} must reject invalid correlation evidence`,
    );
  }

  const linkedEvents = fixture.events.filter((event) =>
    Object.hasOwn(event, "policy_decision_id"),
  );
  assert.ok(linkedEvents.length >= 6);
  for (const event of linkedEvents) {
    assert.equal(event.policy_decision_id, fixture.chain.policy_decision_id);
  }

  assert.equal(deniedUsage.correlation_id, fixture.chain.correlation_id);
  assert.equal(deniedUsage.request_id, fixture.chain.request_id);
  assert.equal(deniedAudit.correlation_id, fixture.chain.correlation_id);
  assert.equal(
    deniedAudit.policy_decision_id,
    fixture.chain.policy_decision_id,
  );
  const canonicalReferences = new Map(
    fixture.events
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
  );
  assert.equal(canonicalReferences.get("usage_event").id, deniedUsage.id);
  assert.equal(
    canonicalReferences.get("audit_event").canonical_schema_path,
    "contracts/schemas/audit-event.schema.json",
  );
  assert.equal(canonicalReferences.get("audit_event").id, deniedAudit.id);

  const remoteEvidence = fixture.cross_repo_evidence.remote;
  assert.equal(
    remoteEvidence.lineage_model,
    "dispatch_fragment_requires_control_plane_join",
  );
  assert.deepEqual(remoteEvidence.dispatch_payload_lineage, {
    has_correlation_id: true,
    has_request_id: false,
    has_policy_decision_id: false,
    has_working_group_id: true,
  });
  assert.ok(
    remoteEvidence.required_control_plane_join_keys.includes(
      "policy_decision_id",
    ),
    "remote dispatch reconstruction must join policy decision lineage from control-plane records",
  );
  assert.equal(
    fixture.residual_risk.security_review_trigger,
    true,
    "security review must be triggered for audit/approval/gateway boundaries",
  );

  const serialized = JSON.stringify(fixture);
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
    "-----BEGIN",
  ]) {
    assert.ok(
      !serialized.toLowerCase().includes(prohibited),
      `synthetic fixture must not include ${prohibited}`,
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
  const error = openapi.components.schemas.ErrorEnvelope.properties.error;
  const fieldErrors =
    openapi.components.schemas.ErrorEnvelope.properties.error.properties
      .field_errors;
  const allowedKeys = Object.keys(fieldErrors.items.properties).sort();

  assert.equal(fieldErrors.items.additionalProperties, false);
  assert.deepEqual(allowedKeys, ["code", "field", "message_key"]);
  assert.ok(!Object.hasOwn(error.properties, "message"));
  assert.ok(!Object.hasOwn(fieldErrors.items.properties, "message"));
  assert.ok(!Object.hasOwn(fieldErrors.items.properties, "redacted"));
  assert.deepEqual(error.properties.code.enum, [
    "validation_failed",
    "conflict",
    "internal_error",
  ]);
  assert.equal(error.properties.support.properties.redacted.const, true);
  assert.match(
    fieldErrors.description,
    /Do not include request bodies, credentials/,
  );
});

test("server message template contract defines locale-aware resource boundaries", async () => {
  const contract = await readJson(
    "contracts/fixtures/server-message-template-contract.json",
  );
  const requiredParts = ["subject", "title", "body", "action", "accessibility"];
  const sensitiveVariablePattern =
    /(secret|token|credential|password|private|raw|prompt|log|stack|trace)/i;

  assert.deepEqual(contract.locale_precedence, [
    "user",
    "working-group",
    "browser",
    "fallback",
  ]);
  assert.equal(contract.fallback_locale, "en");
  assert.deepEqual(contract.template_parts, requiredParts);
  assert.deepEqual(contract.namespaces.sort(), ["emails", "notifications"]);

  const keys = contract.templates.map((template) => template.key).sort();
  assert.deepEqual(keys, [
    "email.approval.requested",
    "email.failure.summary",
    "notification.assignment.created",
    "notification.run.failed_summary",
  ]);

  for (const template of contract.templates) {
    assert.ok(
      contract.namespaces.includes(template.namespace),
      `${template.key} namespace must be supported`,
    );
    assert.match(
      template.resource_prefix,
      new RegExp(`^${template.namespace}\\.`),
      `${template.key} resource prefix must stay in its namespace`,
    );
    assert.ok(
      ["in_app_notification", "email"].includes(template.channel),
      `${template.key} channel must be explicit`,
    );

    for (const variable of template.user_authored_variables) {
      assert.ok(
        template.variables.includes(variable),
        `${template.key} user-authored variable must be declared`,
      );
    }

    for (const variable of template.redacted_variables) {
      assert.ok(
        template.variables.includes(variable),
        `${template.key} redacted variable must be declared`,
      );
    }

    for (const variable of template.variables) {
      if (!sensitiveVariablePattern.test(variable)) continue;
      assert.ok(
        template.redacted_variables.includes(variable),
        `${template.key}.${variable} must be redacted`,
      );
    }
  }
});
