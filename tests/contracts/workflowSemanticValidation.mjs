import assert from "node:assert/strict";

export function validateWorkflowSemantics(workflow) {
  const approvalGatesById = new Map();
  for (const gate of workflow.approval_gates ?? []) {
    assert.ok(
      !approvalGatesById.has(gate.id),
      `approval gate id must be unique: ${gate.id}`,
    );
    approvalGatesById.set(gate.id, gate);
  }

  for (const job of workflow.jobs ?? []) {
    for (const step of job.steps ?? []) {
      if (step.side_effect?.classification !== "protected") continue;

      const gate = approvalGatesById.get(step.approval_gate_ref);
      assert.ok(
        gate,
        `${job.id}.${step.id} approval_gate_ref must reference an approval gate`,
      );
      assert.equal(
        gate.required_before,
        "protected_side_effect",
        `${job.id}.${step.id} approval gate must be required before protected_side_effect`,
      );
    }
  }
}
