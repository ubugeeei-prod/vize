import assert from "node:assert/strict";
import { asArray, asRecord } from "../../json-value.ts";

/** Admit the historical or already-migrated packet and return the current path. */
export function currentNoDupeKeysPacket(historical: unknown): unknown[] {
  const packet = asArray(structuredClone(historical), "historical lint packet");
  assert.equal(packet.length, 1);
  const messages = asArray(asRecord(packet[0]).messages, "historical lint messages");
  assert.equal(messages.length, 1);
  const diagnostic = asRecord(messages[0]);
  assert.equal(diagnostic.ruleId, "script/no-dupe-keys");
  const previous = "docs/content/rules/type-and-script.md";
  const current = "docs/content/rules/reference/script-no-dupe-keys.md";
  assert.ok(diagnostic.ruleDocsPath === previous || diagnostic.ruleDocsPath === current);
  diagnostic.ruleDocsPath = current;
  return packet;
}

// #12828 made `<output>` a labelled form control, as eslint-plugin-vuejs-accessibility
// does. The authored workspace keeps its unlabelled `<output>`, so its main SFC now
// carries exactly this one warning; every other file and diagnostic stays unchanged.
const outputLabelRule = "a11y/form-control-has-label";
const outputLabelText = "<output> elements must have an associated label";
const outputLabelHelp = [
  "Option 1: Use <label> with for:",
  '  <label for="email">Email</label>',
  '  <input id="email" type="email">',
  "Option 2: Wrap input in label:",
  "  <label>",
  "  Email",
  '  <input type="email">',
  "  </label>",
  "Option 3: Use aria-label:",
  '  <input type="search" aria-label="Search">',
  "Note: Placeholder is NOT a substitute for a label.",
].join("\n");
export const outputTag = "<output>";

/** Zero-based start of the authored `<output>` start tag. */
export interface OutputTagPosition {
  line: number;
  character: number;
}

/** The whole CLI JSON message for the authored `<output>` at `at`. */
export function outputLabelLintMessage(at: OutputTagPosition): Record<string, unknown> {
  return {
    ruleId: outputLabelRule,
    ruleDocsPath: "docs/content/rules/reference/a11y-form-control-has-label.md",
    severity: 1,
    message: `[vize:${outputLabelRule}] ${outputLabelText}`,
    line: at.line + 1,
    column: at.character + 1,
    endLine: at.line + 1,
    endColumn: at.character + 1 + outputTag.length,
    help: outputLabelHelp,
  };
}

/** The whole language-server diagnostic for the authored `<output>` at `at`. */
export function outputLabelDiagnostic(at: OutputTagPosition): Record<string, unknown> {
  return {
    range: {
      start: { line: at.line, character: at.character },
      end: { line: at.line, character: at.character + outputTag.length },
    },
    severity: 2,
    code: outputLabelRule,
    codeDescription: { href: `https://eslint.vuejs.org/rules/${outputLabelRule}.html` },
    source: "vize/lint",
    message: `${outputLabelText}\n\nHelp: ${outputLabelHelp}`,
  };
}
