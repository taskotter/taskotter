import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const root = process.cwd();

async function readJson(relativePath) {
  const source = await readFile(path.join(root, relativePath), "utf8");
  return JSON.parse(source);
}

function validate(schema, value, location = schema.title ?? "value") {
  if (schema.const !== undefined) {
    assert.equal(value, schema.const, `${location} const mismatch`);
  }
  if (schema.enum) {
    assert.ok(schema.enum.includes(value), `${location} must be one of ${schema.enum.join(", ")}`);
  }
  if (schema.type) {
    if (schema.type === "object") {
      assert.equal(typeof value, "object", `${location} must be object`);
      assert.notEqual(value, null, `${location} must not be null`);
      assert.ok(!Array.isArray(value), `${location} must not be array`);
    } else if (schema.type === "array") {
      assert.ok(Array.isArray(value), `${location} must be array`);
    } else if (schema.type === "integer") {
      assert.equal(Number.isInteger(value), true, `${location} must be integer`);
    } else {
      assert.equal(typeof value, schema.type, `${location} must be ${schema.type}`);
    }
  }
  if (schema.pattern && typeof value === "string") {
    assert.match(value, new RegExp(schema.pattern), `${location} pattern mismatch`);
  }
  if (schema.required) {
    for (const key of schema.required) {
      assert.ok(Object.hasOwn(value, key), `${location}.${key} is required`);
    }
  }
  if (schema.additionalProperties === false && schema.properties && value && typeof value === "object" && !Array.isArray(value)) {
    for (const key of Object.keys(value)) {
      assert.ok(Object.hasOwn(schema.properties, key), `${location}.${key} is not allowed`);
    }
  }
  if (schema.properties && value && typeof value === "object" && !Array.isArray(value)) {
    for (const [key, childSchema] of Object.entries(schema.properties)) {
      if (Object.hasOwn(value, key)) validate(resolveRef(schema, childSchema), value[key], `${location}.${key}`);
    }
  }
  if (schema.items && Array.isArray(value)) {
    value.forEach((item, index) => validate(resolveRef(schema, schema.items), item, `${location}[${index}]`));
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

test("runtime fixtures match their canonical schemas", async () => {
  const cases = [
    ["contracts/schemas/policy-decision.schema.json", "contracts/fixtures/policy-decision.allow.runner-job.json"],
    ["contracts/schemas/usage-event.schema.json", "contracts/fixtures/usage-event.gateway-request.json"],
    ["contracts/schemas/audit-event.schema.json", "contracts/fixtures/audit-event.policy-denied.json"]
  ];

  for (const [schemaPath, fixturePath] of cases) {
    validate(await readJson(schemaPath), await readJson(fixturePath));
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
      `${fileName} generated artifact must mirror canonical source`
    );
  }
});

test("OpenAPI exposes the first control-plane resource surface", async () => {
  const openapi = await readJson("contracts/openapi/taskotter-control-plane.openapi.json");
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
    "/audit/events"
  ];

  for (const route of expectedPaths) {
    assert.ok(openapi.paths[route], `${route} must be present`);
  }
  assert.equal(openapi.info.version, "0.1.0");
});
