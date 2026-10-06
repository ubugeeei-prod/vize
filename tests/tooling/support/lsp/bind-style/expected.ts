import assert from "node:assert/strict";
import type { LspRange } from "../protocol.ts";

export type Diagnostic = {
  range: LspRange;
  severity: number;
  code: string;
  codeDescription: { href: string };
  source: string;
  message: string;
};
export type Action = {
  title: string;
  kind: string;
  diagnostics: Diagnostic[];
  isPreferred: boolean;
  edit: { changes: Record<string, Array<{ range: LspRange; newText: string }>> };
};

export const PROP = "vue/prefer-props-shorthand";
export const STYLE = "vue/v-bind-style";
export const HTML = "vue/no-v-html";

export function position(source: string, offset: number) {
  assert.ok(offset >= 0 && offset <= source.length);
  const head = source.slice(0, offset).split("\n");
  return { line: head.length - 1, character: head.at(-1)!.length };
}

export function range(source: string, text: string, occurrence = 0): LspRange {
  let start = -1;
  for (let index = 0; index <= occurrence; index++) start = source.indexOf(text, start + 1);
  assert.ok(start >= 0, `authored carrier missing: ${text}`);
  return { start: position(source, start), end: position(source, start + text.length) };
}

export function diagnostics(source: string): Diagnostic[] {
  const rows: Diagnostic[] = [];
  for (const match of source.matchAll(
    /(?<=\s)(v-bind:title="title"|:title="title")(?=\s|\/|>)/gu,
  )) {
    const carrier = match[0];
    const span = {
      start: position(source, match.index),
      end: position(source, match.index + carrier.length),
    };
    rows.push(
      diagnostic(
        PROP,
        span,
        "Use shorthand syntax for same-name prop binding\n\nHelp: Use shorthand prop syntax",
      ),
    );
    if (carrier.startsWith("v-bind"))
      rows.push(
        diagnostic(
          STYLE,
          span,
          'Prefer shorthand `:` over `v-bind:`\n\nHelp: Use :attr="value" instead of v-bind:attr="value"',
        ),
      );
  }
  rows.push(
    diagnostic(
      HTML,
      range(source, 'v-html="html"'),
      "v-html can lead to XSS attacks. Avoid using it with user-provided content\n\n" +
        "Help: Security Risk: v-html renders raw HTML and can execute malicious scripts from user input.\n" +
        "Alternatives:\n1. Use text interpolation (auto-escaped):\n  <p>{{ userContent }}</p>\n" +
        "2. Use a sanitization library:\n  import DOMPurify from 'dompurify'\n" +
        "  const safeHtml = DOMPurify.sanitize(userInput)\n3. Use a markdown renderer with XSS protection\n" +
        "If you must use v-html:\n- Never use with user-provided content\n- Only use with trusted, static content",
    ),
  );
  return rows;
}

function diagnostic(code: string, range: LspRange, message: string): Diagnostic {
  return {
    range,
    severity: 2,
    code,
    codeDescription: { href: `https://eslint.vuejs.org/rules/${code.replace("vue/", "")}.html` },
    source: "vize/lint",
    message,
  };
}

export function actions(source: string, uri: string, diagnostic: Diagnostic) {
  const code = String(diagnostic.code);
  const line = source.split("\n")[diagnostic.range.start.line];
  const indent = /^\s*/u.exec(line)![0].replace(/\r$/u, "");
  const newline = source.includes("\r\n") ? "\r\n" : "\n";
  const insert = { line: diagnostic.range.start.line, character: 0 };
  const rows: Action[] = [];
  if (code !== HTML) {
    const fixRange =
      code === STYLE
        ? diagnostic.range
        : {
            start: {
              ...diagnostic.range.end,
              character: diagnostic.range.end.character - '="title"'.length,
            },
            end: diagnostic.range.end,
          };
    rows.push({
      title: code === STYLE ? "Fix: Use shorthand syntax" : "Fix: Use shorthand prop syntax",
      kind: "quickfix",
      diagnostics: [diagnostic],
      isPreferred: true,
      edit: {
        changes: { [uri]: [{ range: fixRange, newText: code === STYLE ? ':title="title"' : "" }] },
      },
    });
  }
  rows.push({
    title: `Suppress with @vize:forget (${code})`,
    kind: "quickfix",
    diagnostics: [diagnostic],
    isPreferred: false,
    edit: {
      changes: {
        [uri]: [
          {
            range: { start: insert, end: insert },
            newText: `${indent}<!-- @vize:forget ${code} -->${newline}`,
          },
        ],
      },
    },
  });
  return rows;
}

export function apply(source: string, action: Action, uri: string): string {
  assert.deepEqual(Object.keys(action.edit), ["changes"]);
  assert.deepEqual(Object.keys(action.edit.changes), [uri]);
  const edits = action.edit.changes[uri] as Array<{ range: LspRange; newText: string }>;
  assert.equal(edits.length, 1);
  const [{ range, newText }] = edits;
  const offset = (p: { line: number; character: number }) => {
    const lines = source.split("\n");
    assert.ok(p.line < lines.length && p.character <= lines[p.line].length);
    return lines.slice(0, p.line).reduce((sum, line) => sum + line.length + 1, 0) + p.character;
  };
  return source.slice(0, offset(range.start)) + newText + source.slice(offset(range.end));
}
