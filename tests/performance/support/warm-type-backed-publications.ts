import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "../../tooling/support/lsp/assertions.ts";

// Independently authored from the two Patina rule/i18n bodies and PlainText
// help rendering, not from a captured diagnostic response. The original source
// has a static h2 click handler without keyboard events, role or tabindex.
const messages = [
  [
    "a11y/click-events-have-key-events",
    [
      "Non-interactive elements with @click must also have keyboard event handlers",
      "",
      "Help: Why: Keyboard users cannot activate click-only handlers.",
      "Best solution: Use semantic elements:",
      '  <button @click="handleClick">Click me</button>',
      "If you must use a div:",
      "  <div",
      '  role="button"',
      '  tabindex="0"',
      '  @click="handleClick"',
      '  @keydown.enter="handleClick"',
      '  @keydown.space="handleClick"',
      "  >",
      "  Click me",
      "  </div>",
    ].join("\n"),
  ],
  [
    "a11y/no-static-element-interactions",
    [
      "<h2> is a static element and should not have interactive event handlers",
      "",
      "Help: Why: Non-interactive elements (div, span, etc.) should not have click/keyboard handlers without a role.",
      "Fix options:",
      "1. Use a native interactive element:",
      '  <button @click="handle">Click me</button>',
      "2. Add role and tabindex:",
      '  <div role="button" tabindex="0" @click="handle">',
      "  Click me",
      "  </div>",
    ].join("\n"),
  ],
] as const;

export function originalPublications(workspace: string, source: string, promptChanges: boolean) {
  const uri = pathToFileURL(path.join(workspace, "src/c/Comp0.vue")).href;
  const otherUri = pathToFileURL(path.join(`${workspace}-other`, "src/c/Comp0.vue")).href;
  const tag = '<h2 @click="toggle(!open)">';
  const diagnostics = (text: string) => {
    const offset = text.indexOf(tag);
    assert.notEqual(offset, -1);
    assert.equal(text.lastIndexOf(tag), offset);
    return messages.map(([code, message]) => ({
      code,
      codeDescription: { href: `https://eslint.vuejs.org/rules/${code}.html` },
      message,
      range: {
        start: offsetToPosition(text, offset),
        end: offsetToPosition(text, offset + tag.length),
      },
      severity: 2,
      source: "vize/lint",
    }));
  };
  const publish = (document: string, text: string, version?: number) => ({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: document,
      diagnostics: diagnostics(text),
      ...(version == null ? {} : { version }),
    },
  });
  // didOpen sends nonempty unversioned sync feedback, then complete version1.
  const notifications = [
    {
      jsonrpc: "2.0",
      method: "window/logMessage",
      params: { message: "vize_maestro LSP server initialized", type: 3 },
    },
    publish(uri, source),
    publish(uri, source, 1),
  ];
  // The four sequential source edits are awaited to complete before the next.
  // Current didChange sends prompt versioned feedback AND complete feedback.
  const probe = source.replace(
    "const open = ref(false);",
    "const open = ref(false);\nconst configProbe = null;",
  );
  assert.notEqual(probe, source);
  for (const version of [2, 3, 4, 5]) {
    const text = version === 4 ? probe : source;
    if (promptChanges) notifications.push(publish(uri, text, version));
    notifications.push(publish(uri, text, version));
  }
  notifications.push({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri, diagnostics: [] },
  });
  notifications.push(publish(uri, source), publish(uri, source, 1));
  notifications.push(publish(otherUri, source), publish(otherUri, source, 1));
  // Native retirement then the final version2 edit target the ORIGINAL URI.
  if (promptChanges) notifications.push(publish(uri, source, 2));
  notifications.push(publish(uri, source, 2));
  return notifications;
}
