import assert from "node:assert/strict";

export const originalNestedSource =
  "<script setup>const value=1</script><template><i>{{value}}</i></template>";
export const originalNestedRefusal = {
  completeModule: false,
  diagnostics: [],
  name: "nested",
  reason:
    "Ssr(SsrError { node: Some(NodeId(1)), span: Span { start: 49, end: 58 }, kind: Unsupported(Operation) })",
  source: originalNestedSource,
};

export function withoutOriginalNested(rows: any[]) {
  assert.equal(rows.length, 20);
  assert.equal(rows[15].name, "nested");
  assert.equal(rows[15].source, originalNestedSource);
  if (Object.hasOwn(rows[15], "reason")) assert.deepEqual(rows[15], originalNestedRefusal);
  const successor = [...rows.slice(0, 15), ...rows.slice(16)];
  assert.deepEqual([...successor.slice(0, 15), rows[15], ...successor.slice(15)], rows);
  return successor;
}

export function directChildHistoricalSuccessor(packet: any) {
  return { ...packet, refusals: withoutOriginalNested(packet.refusals) };
}
