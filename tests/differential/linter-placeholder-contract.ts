import assert from "node:assert/strict";

// Exact immutable input wires. Production capability never depends on case IDs.
export const ORIGINAL_PLACEHOLDER_WIRES = [
  {
    id: "placeholder-label-unrequired-empty-control",
    history: "dbab039dde930c513dea7d1358eebde04296661a",
    source: '<select><option value="">Any</option></select>',
    filename: "History.vue",
    entry: "template",
    rule: "a11y/placeholder-label-option",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "placeholder-label-required-empty-control",
    history: "dbab039dde930c513dea7d1358eebde04296661a",
    source: '<select required><option value="">Choose</option></select>',
    filename: "History.vue",
    entry: "template",
    rule: "a11y/placeholder-label-option",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "placeholder-label-required-nonempty-report",
    history: "dbab039dde930c513dea7d1358eebde04296661a",
    source: '<select required><option value="a">A</option></select>',
    filename: "History.vue",
    entry: "template",
    rule: "a11y/placeholder-label-option",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
  {
    id: "placeholder-label-optgroup-report",
    history: "dbab039dde930c513dea7d1358eebde04296661a",
    source:
      '<select required><optgroup label="G"><option value="">Choose</option></optgroup></select>',
    filename: "History.vue",
    entry: "template",
    rule: "a11y/placeholder-label-option",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
];

export function expectedPlaceholderReason(input: unknown, api: string) {
  assert(input !== null && typeof input === "object" && !Array.isArray(input));
  const id = "id" in input ? input.id : undefined;
  const original = ORIGINAL_PLACEHOLDER_WIRES.find((wire) => wire.id === id);
  assert(original, "only the four whole original placeholder wires are registered");
  assert.deepEqual(input, original, "complete original placeholder input/options must stay exact");
  assert.equal(api, "--native-current-api");
  if (original.id !== "placeholder-label-optgroup-report") return null;
  return {
    api,
    entry: "template",
    kind: "UnsupportedContext",
    detail: "UnsupportedContext { span: Span { start: 17, end: 37 } }",
  };
}
