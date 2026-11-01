import assert from "node:assert/strict";

// These exact immutable original wires constrain this pack's expected outcome.
// Production admission remains the genuine registered native callback/owner.
export const ORIGINAL_INPUT_LABEL_WIRES = [
  {
    id: "form-label-hidden-input-report-9751",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: '<input hidden type="file" />',
    filename: "HiddenInput.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "form-label-hidden-select-control",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: "<select hidden><option>A</option></select>",
    filename: "HiddenSelect.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "form-label-hidden-textarea-control",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: "<textarea hidden></textarea>",
    filename: "HiddenTextarea.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 0,
    fixes: 0,
  },
  {
    id: "form-label-until-found-control",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: '<input hidden="until-found" type="text" />',
    filename: "UntilFoundInput.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
  {
    id: "form-label-bound-hidden-control",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: '<input :hidden="collapsed" type="text" />',
    filename: "BoundHiddenInput.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
  {
    id: "form-label-visible-input-control",
    history: "708c7fd03f9cfd399d81a246d016abda7e095929",
    source: '<input type="text" />',
    filename: "VisibleInput.vue",
    entry: "template",
    rule: "a11y/form-control-has-label",
    vue_version: null,
    vapor: null,
    diagnostics: 1,
    fixes: 0,
  },
];

export function expectedInputLabelReason(input: unknown, api: string) {
  assert(
    input !== null && typeof input === "object" && !Array.isArray(input),
    "the original form-label wire must be an object",
  );
  const id = "id" in input ? input.id : undefined;
  const original = ORIGINAL_INPUT_LABEL_WIRES.find((wire) => wire.id === id);
  assert(original, "only the six whole original form-label wires are registered");
  assert.deepEqual(input, original, "complete original form-label input/options must stay exact");
  if (original.source.startsWith("<input") || original.source.startsWith("<select")) return null;
  const end = original.source.indexOf(">") + 1;
  return {
    api,
    entry: "template",
    kind: "UnsupportedContext",
    detail: `UnsupportedContext { span: Span { start: 0, end: ${end} } }`,
  };
}
