import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { symlinkDirectory } from "../../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../../_helpers/realworld-typecheck.ts";
import { isDiagnosticsForUri } from "../../tooling/support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "../../tooling/support/lsp/protocol.ts";
import { LspSession } from "../../tooling/support/lsp/session.ts";
import { root, testOutputRoot } from "../../tooling/support/lsp/paths.ts";

// LSP companion to snapshots/check/class-component.ts (#2971 audit item 7):
// the batch suite proves vue-class-component / vue-property-decorator member
// resolution for `vize check`, this oracle proves the same fixture survives a
// didOpen-clean -> didChange-broken -> didChange-repaired editor cycle without
// a server restart. The broken edit typos the `@Prop` usage inside the
// `greeting` getter (`this.name` -> `this.nam`), so the diagnostic must
// resolve against the class instance type at the exact authored range.
const fixtureDir = path.join(root, "tests/_fixtures/_projects/class-component");
const fixtureSources = ["App.vue", "HelloDecorator.vue", "TypeErrorDecorator.vue"];
const requiredPackages = ["vue", "vue-class-component", "vue-property-decorator"];
const cleanPropUsage = "Hello, ${this.name}!";
const brokenPropUsage = "Hello, ${this.nam}!";

const brokenPropUsageDiagnostic = {
  range: {
    start: { line: 13, character: 26 },
    end: { line: 13, character: 29 },
  },
  severity: 1,
  code: 2551,
  source: "vize/types",
  message:
    "Property 'nam' does not exist on type 'HelloDecorator'. Did you mean 'name'?" +
    "\n\nIf you intended to read the reactive value, try `.value`. (vize/types)",
};

test("class-component @Prop usage typo breaks and repairs over didChange", async () => {
  const corsaPath = resolveTsgoBinary();
  const workspaceDir = createWorkspace(corsaPath);
  const helloPath = path.join(workspaceDir, "src/HelloDecorator.vue");
  const helloUri = pathToFileURL(helloPath).href;
  const cleanSource = fs.readFileSync(helloPath, "utf8");
  const brokenSource = applyExactEdit(cleanSource, cleanPropUsage, brokenPropUsage);

  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });
    const serverPid = session.processId;

    session.notify("textDocument/didOpen", {
      textDocument: { uri: helloUri, languageId: "vue", version: 1, text: cleanSource },
    });
    assert.deepEqual(await waitForDiagnostics(session, helloUri, 1), {
      diagnostics: [],
      uri: helloUri,
      version: 1,
    });

    session.notify("textDocument/didChange", {
      textDocument: { uri: helloUri, version: 2 },
      contentChanges: [{ text: brokenSource }],
    });
    assert.deepEqual(await waitForDiagnostics(session, helloUri, 2), {
      diagnostics: [brokenPropUsageDiagnostic],
      uri: helloUri,
      version: 2,
    });

    session.notify("textDocument/didChange", {
      textDocument: { uri: helloUri, version: 3 },
      contentChanges: [{ text: cleanSource }],
    });
    assert.deepEqual(await waitForDiagnostics(session, helloUri, 3), {
      diagnostics: [],
      uri: helloUri,
      version: 3,
    });

    assert.equal(session.processId, serverPid, "one server must serve the whole cycle");
    assert.equal(
      fs.readFileSync(helloPath, "utf8"),
      cleanSource,
      "didChange cycles must never touch the on-disk source",
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

// Usage-site `@Prop` contracts (#3298; the gap this oracle originally
// documented as skipped). A `@Prop`-decorated member is a declared prop, so a
// parent gets the same diagnostics a `defineProps` child produces:
//
//   - dropping the required prop (`<HelloDecorator nme="World" />`) is a
//     `component-required-props` error on the tag name, joined by the type
//     layer's TS2353 on the unknown attr key: `nme` is not a declared prop, so
//     strict component-attr checking points at the unsupported spelling while
//     the component diagnostic keeps the missing required prop at the tag, and
//   - binding a mismatched type (`<HelloDecorator :name="123" />`) is a TS2322
//     on the attribute name.
//
// Both are asserted through the same didOpen-clean -> didChange-broken ->
// didChange-repaired cycle as the member typo above, so a stale contract cached
// across edits would fail here too.
const cleanUsage = '<HelloDecorator name="World" />';
const brokenUsage = '<HelloDecorator nme="World" />\n    <HelloDecorator :name="123" />';

const missingRequiredPropDiagnostic = {
  range: {
    start: { line: 19, character: 5 },
    end: { line: 19, character: 19 },
  },
  severity: 1,
  code: "component-required-props",
  codeDescription: {
    href: "https://vuejs.org/guide/components/props.html#prop-validation",
  },
  source: "vize/components",
  message:
    "<HelloDecorator> is missing required prop: `name`\n\nPass the prop in this " +
    "template usage, or make it optional/provide a default in the child component.",
};

const unknownPropTypeDiagnostic = {
  range: {
    start: { line: 19, character: 20 },
    end: { line: 19, character: 23 },
  },
  severity: 1,
  code: 2353,
  source: "vize/types",
  message:
    "Object literal may only specify known properties, and '\"nme\"' does not exist in type " +
    '\'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & { "aria-activedescendant"?: ' +
    'unknown; "aria-atomic"?: unknown; "aria-autocomplete"?: unknown; "aria-busy"?: ' +
    'unknown; "aria-checked"?: unknown; "aria-colcount"?: unknown; ... 195 more ...; ' +
    "ref_key?: unknown; } & __VizeCustomDataFallthroughAttrs & Partial<.....'.",
};

const propTypeMismatchDiagnostic = {
  range: {
    start: { line: 20, character: 21 },
    end: { line: 20, character: 25 },
  },
  severity: 1,
  code: 2322,
  source: "vize/types",
  message: "Type 'number' is not assignable to type 'string'.",
};

test("class-component usage sites enforce @Prop contracts", async () => {
  const corsaPath = resolveTsgoBinary();
  const workspaceDir = createWorkspace(corsaPath);
  const appPath = path.join(workspaceDir, "src/App.vue");
  const appUri = pathToFileURL(appPath).href;
  const cleanSource = fs.readFileSync(appPath, "utf8");
  const brokenSource = applyExactEdit(cleanSource, cleanUsage, brokenUsage);

  const session = new LspSession();
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: true });

    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: cleanSource },
    });
    assert.deepEqual(await waitForDiagnostics(session, appUri, 1), {
      diagnostics: [],
      uri: appUri,
      version: 1,
    });

    session.notify("textDocument/didChange", {
      textDocument: { uri: appUri, version: 2 },
      contentChanges: [{ text: brokenSource }],
    });
    assert.deepEqual(await waitForDiagnostics(session, appUri, 2), {
      diagnostics: [
        missingRequiredPropDiagnostic,
        unknownPropTypeDiagnostic,
        propTypeMismatchDiagnostic,
      ],
      uri: appUri,
      version: 2,
    });

    session.notify("textDocument/didChange", {
      textDocument: { uri: appUri, version: 3 },
      contentChanges: [{ text: cleanSource }],
    });
    assert.deepEqual(await waitForDiagnostics(session, appUri, 3), {
      diagnostics: [],
      uri: appUri,
      version: 3,
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
  }
});

async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
): Promise<PublishDiagnosticsParams> {
  const params = (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    120_000,
  )) as PublishDiagnosticsParams;
  return {
    ...params,
    diagnostics: params.diagnostics.filter((diagnostic) => !isLegacyDecoratorNoise(diagnostic)),
  };
}

function isLegacyDecoratorNoise(
  diagnostic: PublishDiagnosticsParams["diagnostics"][number],
): boolean {
  return (
    diagnostic.source === "vize/types" &&
    diagnostic.code === 1240 &&
    diagnostic.message.includes("Unable to resolve signature of property decorator")
  );
}

/**
 * Copies the checked-in fixture into a disposable workspace so the oracle can
 * point `typeChecker.corsaPath` at the hydrated tsgo binary without mutating
 * the fixture. Runtime packages are symlinked from tests/node_modules, where
 * tests/package.json pins the same versions the fixture declares.
 */
function createWorkspace(corsaPath: string): string {
  const outputRoot = path.join(testOutputRoot, "class-component-lsp-oracle");
  fs.mkdirSync(outputRoot, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(outputRoot, "workspace-"));

  fs.mkdirSync(path.join(workspaceDir, "src"), { recursive: true });
  for (const source of fixtureSources) {
    fs.copyFileSync(path.join(fixtureDir, "src", source), path.join(workspaceDir, "src", source));
  }

  const packageRoot = path.join(root, "tests/node_modules");
  for (const packageName of requiredPackages) {
    const packageDir = path.join(packageRoot, packageName);
    assert.ok(fs.existsSync(packageDir), `${packageName} must be installed in tests/node_modules`);
    symlinkDirectory(packageDir, path.join(workspaceDir, "node_modules", packageName));
  }
  const vueNamespace = path.join(packageRoot, "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlinkDirectory(vueNamespace, path.join(workspaceDir, "node_modules/@vue"));
  }

  // Mirrors the fixture tsconfig, minus the `paths` remap it needs to reach
  // tests/node_modules from its checked-in location.
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    json({
      compilerOptions: {
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        strict: true,
        skipLibCheck: true,
        noEmit: true,
        experimentalDecorators: true,
        lib: ["ES2022", "DOM", "DOM.Iterable"],
      },
      include: ["src/**/*.vue"],
    }),
  );
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    json({ typeChecker: { corsaPath } }),
  );
  return workspaceDir;
}

function applyExactEdit(source: string, expected: string, replacement: string): string {
  const first = source.indexOf(expected);
  assert.notEqual(first, -1, `fixture does not contain the expected edit anchor: ${expected}`);
  assert.equal(source.indexOf(expected, first + expected.length), -1, "edit anchor must be unique");
  return `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
