import assert from "node:assert/strict";
import { createHmac, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const root = process.cwd();

async function readJson(relativePath) {
  const source = await readFile(path.join(root, relativePath), "utf8");
  return JSON.parse(source);
}

function serializeBody(body) {
  return JSON.stringify(body);
}

function signBody({ body, key, timestamp }) {
  const digest = createHmac("sha256", key)
    .update(`${timestamp}.${serializeBody(body)}`)
    .digest("hex");
  return `sha256=${digest}`;
}

function signaturesMatch(received, expected) {
  if (!received?.startsWith("sha256=")) return false;

  const receivedDigest = Buffer.from(received.slice("sha256=".length), "hex");
  const expectedDigest = Buffer.from(expected.slice("sha256=".length), "hex");
  if (receivedDigest.length !== expectedDigest.length) return false;

  return timingSafeEqual(receivedDigest, expectedDigest);
}

function makeRequest(fixture, fixtureCase) {
  const timestamp = fixtureCase.timestamp ?? fixture.now_epoch_seconds;
  return {
    headers: {
      [fixture.timestamp_header]: String(timestamp),
      [fixture.signature_header]:
        fixtureCase.signature_override ??
        signBody({
          body: fixtureCase.body,
          key: fixture.fake_key_value,
          timestamp,
        }),
    },
    body: fixtureCase.body,
  };
}

function reject(status, code, stageLog) {
  return {
    status,
    code,
    error: "Webhook request rejected.",
    stageLog,
  };
}

function processWebhook({ fixture, fixtureCase, sideEffects }) {
  const stageLog = ["request_received"];
  const request = makeRequest(fixture, fixtureCase);
  const timestamp = Number(request.headers[fixture.timestamp_header]);
  const expectedSignature = signBody({
    body: request.body,
    key: fixture.fake_key_value,
    timestamp,
  });

  stageLog.push("signature_verified");
  if (
    !signaturesMatch(
      request.headers[fixture.signature_header],
      expectedSignature,
    )
  ) {
    return reject(401, "webhook_signature_invalid", stageLog);
  }

  stageLog.push("freshness_verified");
  if (
    !Number.isInteger(timestamp) ||
    Math.abs(fixture.now_epoch_seconds - timestamp) >
      fixture.freshness_window_seconds
  ) {
    return reject(401, "webhook_timestamp_stale", stageLog);
  }

  const seenNonces = new Set(fixtureCase.preseen_nonces ?? []);
  const seenDeliveries = new Set(fixtureCase.preseen_delivery_ids ?? []);

  stageLog.push("replay_checked");
  if (seenNonces.has(request.body.nonce)) {
    return reject(409, "webhook_nonce_replayed", stageLog);
  }
  if (seenDeliveries.has(request.body.delivery_id)) {
    return reject(409, "webhook_delivery_replayed", stageLog);
  }

  stageLog.push("event_type_checked");
  if (!fixture.supported_event_types.includes(request.body.event_type)) {
    return reject(400, "webhook_event_unsupported", stageLog);
  }

  stageLog.push("workflow_target_authorized");
  if (!fixture.allowed_workflow_targets.includes(request.body.workflow_id)) {
    return reject(403, "webhook_target_denied", stageLog);
  }

  stageLog.push("side_effect_executed");
  sideEffects.push({
    delivery_id: request.body.delivery_id,
    event_id: request.body.event_id,
    workflow_id: request.body.workflow_id,
  });
  seenNonces.add(request.body.nonce);
  seenDeliveries.add(request.body.delivery_id);

  return {
    status: 202,
    code: "webhook_accepted",
    stageLog,
  };
}

test("webhook signature, freshness, replay, event, and workflow target fixtures are enforced before side effects", async () => {
  const fixture = await readJson(
    "contracts/fixtures/webhook-trigger-security-cases.json",
  );
  const sideEffectingCaseNames = new Set([
    "valid_request",
    "allowed_workflow_target",
  ]);

  for (const fixtureCase of fixture.cases) {
    const sideEffects = [];
    const result = processWebhook({ fixture, fixtureCase, sideEffects });

    assert.equal(
      result.status,
      fixtureCase.expected_status,
      `${fixtureCase.name} status`,
    );

    if (sideEffectingCaseNames.has(fixtureCase.name)) {
      assert.equal(sideEffects.length, 1, `${fixtureCase.name} side effect`);
      assert.ok(
        result.stageLog.indexOf("signature_verified") <
          result.stageLog.indexOf("freshness_verified"),
        `${fixtureCase.name} verifies freshness only after signature check`,
      );
      assert.ok(
        result.stageLog.indexOf("freshness_verified") <
          result.stageLog.indexOf("replay_checked"),
        `${fixtureCase.name} checks replay only after freshness`,
      );
      assert.ok(
        result.stageLog.indexOf("replay_checked") <
          result.stageLog.indexOf("side_effect_executed"),
        `${fixtureCase.name} runs side effect only after replay check`,
      );
    } else {
      assert.equal(
        sideEffects.length,
        0,
        `${fixtureCase.name} must not run side effects`,
      );
      assert.ok(
        !result.stageLog.includes("side_effect_executed"),
        `${fixtureCase.name} rejected before side effect stage`,
      );
    }
  }
});

test("webhook rejection errors do not expose hidden workflow or resource details", async () => {
  const fixture = await readJson(
    "contracts/fixtures/webhook-trigger-security-cases.json",
  );
  const deniedTarget = fixture.cases.find(
    (fixtureCase) => fixtureCase.name === "denied_workflow_target",
  );

  const result = processWebhook({
    fixture,
    fixtureCase: deniedTarget,
    sideEffects: [],
  });
  const serializedResult = JSON.stringify(result);

  assert.equal(result.status, 403);
  assert.equal(result.error, "Webhook request rejected.");
  for (const fragment of deniedTarget.hidden_resource_fragments) {
    assert.doesNotMatch(serializedResult, new RegExp(fragment));
  }
});

test("webhook security fixtures use generated fake key material only", async () => {
  const fixture = await readJson(
    "contracts/fixtures/webhook-trigger-security-cases.json",
  );
  const serialized = JSON.stringify(fixture);

  assert.equal(fixture.fake_key_id, "fake_webhook_key_01");
  assert.match(fixture.fake_key_value, /^fake-/);
  for (const forbidden of [
    /whsec_/i,
    /sk_live/i,
    /rk_live/i,
    /ghp_[a-z0-9_]+/i,
    /bearer\s+[a-z0-9._-]+/i,
    /-----BEGIN [A-Z ]+PRIVATE KEY-----/,
  ]) {
    assert.doesNotMatch(serialized, forbidden);
  }
});
