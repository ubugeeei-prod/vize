// Injection planning and application for the seeded-defect generator
// (Davinci P0-13). Two pilot defect classes:
//
//   class (a) "undefined-template-ref": rename one `<script setup>` binding
//   (first eligible, deterministic — see fpfn-seed-sfc.mjs for the
//   eligibility contract) everywhere in the script block, so the template
//   reference dangles. Expected diagnostic: vue/no-undefined-refs.
//
//   class (b) "unused-binding": inject `const __davinci_seeded_unused = 0;`
//   into `<script setup>` (creating the block when the file has none).
//   Expected diagnostic: none — vize_croquis computes `unused_bindings` but
//   no lint rule consumes it, which is exactly the documented FN this pilot
//   records (davinci-road/plan/ledger-fn.md).
//
// Every edit is recorded with its original-file span and length delta so
// the identity assertion can map pristine-run diagnostics into seeded-file
// coordinates exactly (no fuzzy matching anywhere).

import {
  blankScriptNoise,
  escapeRegExp,
  extractBlocks,
  identifierOccurrences,
  isShadowedInTemplate,
  templateExpressionSegments,
  topLevelBindings,
  totalTemplateOccurrences,
} from "./fpfn-seed-sfc.mjs";
import { indexToLineCol, lineStartsOf } from "./fpfn-shared.mjs";

export const CLASS_A = "undefined-template-ref";
export const CLASS_B = "unused-binding";
export const CLASS_A_RULE = "vue/no-undefined-refs";
export const SEEDED_NAME_SUFFIX = "__davinci_seeded";
export const UNUSED_BINDING_NAME = "__davinci_seeded_unused";
export const UNUSED_BINDING_STATEMENT = `const ${UNUSED_BINDING_NAME} = 0;\n`;

/** Standalone-token occurrences of `name` in blanked script text. */
function scriptTokenOccurrences(scriptContent, name) {
  const blanked = blankScriptNoise(scriptContent);
  const spans = [];
  let unsure = false;
  const tokenRe = new RegExp(escapeRegExp(name), "g");
  for (const match of blanked.matchAll(tokenRe)) {
    const at = match.index;
    const before = at === 0 ? "" : blanked[at - 1];
    const after = blanked[at + name.length] ?? "";
    if (/[A-Za-z0-9_$]/.test(before) || /[A-Za-z0-9_$]/.test(after)) continue;
    if (before === "." || /^\s*:(?!:)/.test(blanked.slice(at + name.length))) {
      // Member access or object-key-shaped occurrence: renaming it is not a
      // plain binding rename, so the binding is ineligible outright.
      unsure = true;
      continue;
    }
    spans.push(at);
  }
  return { spans, unsure };
}

/** Class-(a) plan for one file, or null with a reason when ineligible. */
export function planClassA(source) {
  const blocks = extractBlocks(source);
  if (!blocks.scriptSetup || !blocks.template) {
    return { plan: null, reason: "no-single-script-setup-and-template" };
  }
  const { scriptSetup, template } = blocks;
  for (const name of topLevelBindings(scriptSetup.content)) {
    const seededName = `${name}${SEEDED_NAME_SUFFIX}`;
    if (source.includes(seededName)) continue;
    if (totalTemplateOccurrences(template.content, name) !== 1) continue;
    if (isShadowedInTemplate(template.content, name)) continue;
    const segments = templateExpressionSegments(template.content);
    const hits = [];
    for (const segment of segments) {
      for (const offset of identifierOccurrences(segment.text, name)) {
        hits.push(template.contentStart + segment.start + offset);
      }
    }
    if (hits.length !== 1) continue;
    const script = scriptTokenOccurrences(scriptSetup.content, name);
    if (script.unsure || script.spans.length === 0) continue;
    return {
      plan: {
        name,
        seededName,
        renameSpans: script.spans.map((offset) => [
          scriptSetup.contentStart + offset,
          scriptSetup.contentStart + offset + name.length,
        ]),
        templateRef: [hits[0], hits[0] + name.length],
      },
      reason: null,
    };
  }
  return { plan: null, reason: "no-eligible-binding" };
}

/** Class-(b) plan for one file (always eligible unless already seeded). */
export function planClassB(source) {
  if (source.includes(UNUSED_BINDING_NAME)) return { plan: null, reason: "already-seeded" };
  const blocks = extractBlocks(source);
  if (blocks.scriptSetup) {
    const { content, contentStart } = blocks.scriptSetup;
    const insertAt = contentStart + (content.startsWith("\n") ? 1 : 0);
    return {
      plan: { insertAt, insertText: UNUSED_BINDING_STATEMENT, createdBlock: false },
      reason: null,
    };
  }
  return {
    plan: {
      insertAt: 0,
      insertText: `<script setup>\n${UNUSED_BINDING_STATEMENT}</script>\n\n`,
      createdBlock: true,
    },
    reason: null,
  };
}

/**
 * Apply plans to `source`. Returns the seeded text plus the ascending edit
 * list ({span: [start, end], delta}, original-file coordinates).
 */
export function applySeed(source, classAPlan, classBPlan) {
  const edits = [];
  if (classAPlan) {
    for (const [start, end] of classAPlan.renameSpans) {
      edits.push({
        span: [start, end],
        delta: classAPlan.seededName.length - classAPlan.name.length,
        insert: classAPlan.seededName,
      });
    }
  }
  if (classBPlan) {
    edits.push({
      span: [classBPlan.insertAt, classBPlan.insertAt],
      delta: classBPlan.insertText.length,
      insert: classBPlan.insertText,
    });
  }
  edits.sort((a, b) => a.span[0] - b.span[0] || a.span[1] - b.span[1]);
  let seeded = source;
  for (let i = edits.length - 1; i >= 0; i -= 1) {
    const { span, insert } = edits[i];
    seeded = seeded.slice(0, span[0]) + insert + seeded.slice(span[1]);
  }
  return { seeded, edits: edits.map(({ span, delta }) => ({ span, delta })) };
}

/**
 * Map an original-file offset into the seeded file through the edit list.
 * `isEnd` marks exclusive span ends (they stay put at pure-insert points).
 * Returns {offset, overlap}.
 */
export function mapOffsetThroughEdits(offset, edits, isEnd) {
  let mapped = offset;
  for (const { span, delta } of edits) {
    const [start, end] = span;
    if (start === end) {
      if (offset > start || (offset === start && !isEnd)) mapped += delta;
    } else if (offset >= end) {
      mapped += delta;
    } else if (offset > start) {
      return { offset: mapped, overlap: true };
    }
  }
  return { offset: mapped, overlap: false };
}

/** Whether the original-coordinate span [start, end) crosses any edit. */
export function spanOverlapsEdits(start, end, edits) {
  for (const { span } of edits) {
    const editStart = span[0];
    const editEnd = span[1];
    if (editStart === editEnd) continue; // pure insert: no original text touched
    if (start < editEnd && end > editStart) return true;
  }
  return false;
}

/** Locate a seeded span in the seeded text and attach line/column data. */
export function describeSeededSpan(seededText, lineStarts, start, end) {
  const from = indexToLineCol(seededText, lineStarts, start);
  const to = indexToLineCol(seededText, lineStarts, end);
  return {
    span: [start, end],
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
  };
}

/** Map an original span forward and describe it in seeded coordinates. */
export function describeMappedSpan(seededText, edits, originalStart, originalEnd) {
  const start = mapOffsetThroughEdits(originalStart, edits, false);
  const end = mapOffsetThroughEdits(originalEnd, edits, true);
  if (start.overlap || end.overlap) return null;
  const lineStarts = lineStartsOf(seededText);
  return describeSeededSpan(seededText, lineStarts, start.offset, end.offset);
}
