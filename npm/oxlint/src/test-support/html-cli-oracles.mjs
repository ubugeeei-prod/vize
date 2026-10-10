import fs from "node:fs";
import assert from "node:assert/strict";
import path from "node:path";
import { formatPatinaMessage } from "../format.ts";

export function readEvents(file) {
  return fs.existsSync(file)
    ? fs
        .readFileSync(file, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line))
    : [];
}

export function plainHostEnvironment(original) {
  const environment = { ...original, FORCE_COLOR: "0", NO_COLOR: "1" };
  for (const key of [
    "CI",
    "GITHUB_ACTIONS",
    "AI_AGENT",
    "CURSOR_AGENT",
    "CLAUDECODE",
    "CLAUDE_CODE",
    "REPL_ID",
    "GEMINI_CLI",
    "CODEX_SANDBOX",
    "CODEX_THREAD_ID",
    "COPILOT_CLI",
    "OPENCODE",
    "JUNIE_DATA",
    "JUNIE_SHIM_PATH",
    "EDITOR",
    "TERM_PROGRAM",
  ])
    delete environment[key];
  return environment;
}

export const snapshot = (root) => {
  const result = [];
  const visit = (file) => {
    const stat = fs.lstatSync(file),
      relative = path.relative(root, file);
    if (stat.isSymbolicLink()) result.push([relative, "symlink", fs.readlinkSync(file)]);
    else if (stat.isDirectory()) {
      result.push([relative, "directory"]);
      for (const child of fs.readdirSync(file).sort()) visit(path.join(file, child));
    } else result.push([relative, "file", fs.readFileSync(file).toString("base64")]);
  };
  visit(root);
  return result;
};
export const fixtures = [
  {
    name: "html-warning",
    implicitDefault: true,
    files: { "src/Live.html": "Warning.html" },
    selected: { "src/Live.html": "warning" },
  },
  {
    name: "html-error",
    files: { "src/Live.htm": "Error.htm" },
    selected: { "src/Live.htm": "duplicate-error" },
    rules: { "vize/vue/no-duplicate-attributes": "error" },
    status: 1,
  },
  {
    name: "html-clean",
    files: { "src/Clean.html": "Clean.html" },
    selected: { "src/Clean.html": "empty" },
  },
  {
    name: "literal-original-standalone",
    files: { "src/Standalone.html": "OriginalStandalone.html.txt" },
    selected: { "src/Standalone.html": "empty" },
  },
  {
    name: "original-standalone-options-api",
    files: { "Standalone.html": "OriginalStandalone.html.txt" },
    selected: { "Standalone.html": "original-options-api" },
    rules: { "no-unused-vars": "off", "vize/script/no-options-api": "error" },
    settings: { vize: { helpLevel: "none", preset: "opinionated" } },
    target: "Standalone.html",
    status: 1,
  },
  {
    name: "literal-mixed-originals",
    files: {
      "src/AppPanel.vue": "OriginalAppPanel.vue.txt",
      "src/Standalone.html": "OriginalStandalone.html.txt",
      "src/Live.html": "Warning.html",
      "dist/Built.html": "Warning.html",
      "vendor/Vendor.html": "Warning.html",
      "src/Skipped.html": "Warning.html",
    },
    selected: { "src/Live.html": "warning", "src/Standalone.html": "empty" },
    original: true,
    ignored: true,
  },
  {
    name: "all-html-excluded",
    files: {
      "dist/Built.html": "Warning.html",
      "vendor/Vendor.html": "Warning.html",
      "src/Skipped.html": "Warning.html",
    },
    selected: {},
    ignored: true,
    status: 1,
  },
  {
    name: "deny-html-warning",
    files: { "src/Live.html": "Warning.html" },
    selected: { "src/Live.html": "warning" },
    deny: true,
    status: 1,
  },
  {
    name: "root-deny-html-warning",
    files: { "src/Live.html": "Warning.html" },
    selected: { "src/Live.html": "warning" },
    denyWarnings: true,
    status: 1,
  },
  {
    name: "real-host-plugin-failure",
    files: { "src/Live.html": "Warning.html" },
    selected: {},
    badPlugin: true,
    status: 1,
    refused: true,
  },
  {
    name: "real-implicit-agent-host-refusal",
    files: { "src/Live.html": "Warning.html" },
    selected: {},
    formats: ["default"],
    implicitDefault: true,
    agent: true,
    status: 1,
    refused: true,
  },
];

export function completeCase(capture, nativeCalls, hostPhases) {
  capture.qualified.cases++;
  capture.qualified.wholeNativeCalls += nativeCalls.length;
  capture.qualified.wholeHostPhases += hostPhases.length;
}

export function finishCapture(capture) {
  assert.deepEqual(capture.qualified, {
    cases: 41,
    formats: ["default", "json", "unix", "stylish"],
    wholeNativeCalls: 36,
    wholeHostPhases: 57,
  });
  capture.complete = true;
}

export function expectedOriginalRows(fixture, packet, root, stockRows) {
  if (!fixture.original) return stockRows;
  // Literal bytes 79..92 are v-html="html" in the unchanged OriginalAppPanel.
  return expectedRows(
    "src/AppPanel.vue",
    {
      ...packet,
      diagnostics: packet.diagnostics.map((finding) => ({ ...finding, start: 79, end: 92 })),
    },
    fs.readFileSync(path.join(root, "src/AppPanel.vue")),
  );
}

export function assertStandaloneOutput(fixture, format, output, repository) {
  if (fixture.name === "original-standalone-options-api" && format === "stylish")
    assert.equal(
      output.trim(),
      fs
        .readFileSync(
          path.join(repository, "npm/oxlint/src/__snapshots__/stylish-standalone-html-output.txt"),
          "utf8",
        )
        .trim(),
    );
}

export function expectedRows(file, packet, source, helpLevel = "full") {
  const span = (start, end) => {
    const prefix = source
      .subarray(0, start)
      .toString("utf8")
      .split(/\r\n|\r|\n/u);
    return {
      offset: start,
      length: end - start,
      line: prefix.length,
      column: Buffer.byteLength(prefix.at(-1)) + 1,
    };
  };
  return packet.diagnostics.map((finding) => {
    return {
      message: formatPatinaMessage(finding, {
        hasMappedLocation: true,
        blockLabel: "html",
        helpLevel,
      }),
      code: `vize(${finding.rule_name})`,
      severity: finding.severity,
      filename: file,
      labels: [
        { span: span(finding.start, finding.end) },
        ...finding.labels.map((label) => ({
          label: label.message,
          span: span(label.start, label.end),
        })),
      ],
    };
  });
}
