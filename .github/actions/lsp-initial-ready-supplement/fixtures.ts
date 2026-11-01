import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
export const VBEN_REVISION = "748b182e440863711dc9624924e92440f5ddc4d3";
export const HUGE_NODES = 4096;
export const DOC = "**Invoice total** in minor currency units.";
export const markdown = (type: "number" | "string"): string =>
  "```typescript\n(property) invoiceTotal: " + type + "\n```\n\n" + DOC;
export type Obj = Record<string, unknown>;
export const object = (value: unknown): Obj => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Obj;
};
export const hash = (value: string | Buffer): string =>
  createHash("sha256").update(value).digest("hex");
export function position(source: string, offset: number): { line: number; character: number } {
  assert.ok(offset >= 0 && offset <= source.length);
  const lines = source.slice(0, offset).split("\n");
  return { line: lines.length - 1, character: lines.at(-1)!.length };
}
export function tokenRange(source: string, token: string): Obj {
  const offset = source.lastIndexOf(token);
  assert.ok(offset >= 0);
  return { start: position(source, offset), end: position(source, offset + token.length) };
}
export const supportSource = (): string =>
  `export declare function makeTotals(): {\n  /** ${DOC} */\n  invoiceTotal: number;\n};\nexport declare function makeStringTotals(): {\n  /** ${DOC} */\n  invoiceTotal: string;\n};\nexport interface Ledger {\n${Array.from({ length: HUGE_NODES }, (_, i) => `  field${String(i).padStart(4, "0")}: number;`).join("\n")}\n}\nexport declare function makeLedger(): Ledger;\n`;
export function hugeSource(repaired: boolean): string {
  const rows = Array.from(
    { length: HUGE_NODES },
    (_, i) =>
      `  <div>{{ ledger.${!repaired && i === 0 ? "missingFirst" : !repaired && i === HUGE_NODES - 1 ? "missingFinal" : "field" + String(i).padStart(4, "0")} }}</div>`,
  );
  return `<script setup lang="ts">\nimport { ${repaired ? "makeStringTotals" : "makeTotals"}, makeLedger } from './support';\nconst totals = ${repaired ? "makeStringTotals" : "makeTotals"}();\nconst ledger = makeLedger();\n</script>\n<template>\n${rows.join("\n")}\n  <span>{{ totals.invoiceTotal }}</span>\n</template>\n`;
}
export function expectedDiagnostics(
  source: string,
  kind: "huge4096" | "vben2",
  repaired: boolean,
): Obj[] {
  if (repaired) return [];
  if (kind === "vben2")
    return [
      {
        range: {
          start: position(source, source.indexOf("brokenInvoice")),
          end: position(source, source.indexOf("brokenInvoice") + "brokenInvoice".length),
        },
        severity: 1,
        code: 2322,
        source: "vize/types",
        message: "Type 'string' is not assignable to type 'number'.",
      },
    ];
  return ["missingFirst", "missingFinal"].map((token) => ({
    range: tokenRange(source, token),
    severity: 1,
    code: 2339,
    source: "vize/types",
    message: `Property '${token}' does not exist on type 'Ledger'.`,
  }));
}
export function hoverPacket(source: string, repaired: boolean): Obj {
  return {
    contents: { kind: "markdown", value: markdown(repaired ? "string" : "number") },
    range: tokenRange(source, "invoiceTotal"),
  };
}
export function queryParams(uri: string, source: string): Obj {
  return {
    textDocument: { uri },
    position: object(tokenRange(source, "invoiceTotal")).start,
    context: { triggerKind: 1 },
  };
}
export type Target = { file: string; uri: string; invalid: string; repair: string };
export type Fixture = {
  id: "huge4096" | "vben2";
  directory: string;
  targets: Target[];
  manifest: Obj;
};
export function target(file: string, invalid: string, repair: string): Target {
  return { file, uri: pathToFileURL(file).href, invalid, repair };
}
export function writeHuge(directory: string): Fixture {
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(directory, "support.ts"), supportSource());
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        skipLibCheck: true,
      },
      include: ["*.vue", "*.ts"],
    }),
  );
  const t = target(path.join(directory, "Huge.vue"), hugeSource(false), hugeSource(true));
  fs.writeFileSync(t.file, t.invalid);
  return {
    id: "huge4096",
    directory,
    targets: [t],
    manifest: {
      typedRows: HUGE_NODES,
      additionalNativeProbeExpressions: 1,
      openedTargets: 1,
      checkedTargets: 1,
      closedDependencyFiles: 1,
      sourceBytes: Buffer.byteLength(t.invalid),
      supportBytes: Buffer.byteLength(supportSource()),
    },
  };
}
export function patchVbenLeaf(source: string, repaired: boolean): string {
  const from =
    "import { Fallback } from '@vben/common-ui';\n\ndefineOptions({ name: 'FallbackOfflineDemo' });";
  assert.equal(source.split(from).length, 2);
  assert.equal(source.split('<Fallback status="offline" />').length, 2);
  return source
    .replace(
      from,
      `import { ${repaired ? "makeStringTotals" : "makeTotals"} } from './__native_invoice';\nconst totals = ${repaired ? "makeStringTotals" : "makeTotals"}();\nconst brokenInvoice: number = ${repaired ? "1" : "'wrong'"};`,
    )
    .replace(
      '<Fallback status="offline" />',
      "<div>{{ totals.invoiceTotal }} {{ brokenInvoice }}</div>",
    );
}
export function vbenFixture(directory: string, originals: readonly string[]): Fixture {
  const files = [
    "apps/web-antd/src/views/_core/fallback/offline.vue",
    "apps/web-ele/src/views/_core/fallback/offline.vue",
  ];
  assert.equal(originals.length, 2);
  const targets = files.map((file, i) =>
    target(
      path.join(directory, file),
      patchVbenLeaf(originals[i]!, false),
      patchVbenLeaf(originals[i]!, true),
    ),
  );
  for (const t of targets) {
    fs.writeFileSync(t.file, t.invalid);
    fs.writeFileSync(
      path.join(path.dirname(t.file), "__native_invoice.ts"),
      supportSource().split("export interface Ledger")[0]!,
    );
  }
  return {
    id: "vben2",
    directory,
    targets,
    manifest: {
      fixtureRevision: VBEN_REVISION,
      workspaceInventory: inventory(directory),
      openedTargets: 2,
      checkedTargets: 2,
      closedNativeSupportFiles: 2,
      wholeWorkspaceTypecheck: false,
      originalAndPatched: targets.map((t, i) => ({
        file: files[i],
        originalSha256: hash(originals[i]!),
        invalidSha256: hash(t.invalid),
        repairSha256: hash(t.repair),
      })),
    },
  };
}
export function memoryWorkspace(root: string): { roots: string[]; targets: Target[] } {
  const roots = Array.from({ length: 8 }, (_, i) => path.join(root, `config-${i}`));
  const targets = roots.map((directory) => {
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(
      path.join(directory, "support.ts"),
      supportSource().split("export interface Ledger")[0]!,
    );
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
          skipLibCheck: true,
        },
        include: ["*.vue", "*.ts"],
      }),
    );
    const small = `<script setup lang="ts">\nimport { makeTotals } from './support';\nconst totals = makeTotals();\n</script>\n<template>{{ totals.invoiceTotal }}</template>\n`;
    const t = target(
      path.join(directory, "App.vue"),
      small,
      small.replaceAll("makeTotals", "makeStringTotals"),
    );
    fs.writeFileSync(t.file, t.invalid);
    return t;
  });
  return { roots, targets };
}
export function auditPublications(
  packets: readonly Obj[],
  targets: Target[],
  kind: Fixture["id"],
): void {
  const pending = {
    range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
    severity: 3,
    code: "typecheck-pending",
    source: "vize/types",
    message:
      "Type checking is still running while the shared project warms up. Type diagnostics will appear when it finishes. Change `vize.typecheck.timeoutMs` (initialization option `typecheckTimeoutMs`) or `VIZE_LSP_NATIVE_DIAGNOSTIC_TIMEOUT_MS` to adjust this notification threshold.",
  };
  for (const packet of packets) {
    assert.deepEqual(Object.keys(packet).sort(), ["diagnostics", "uri", "version"]);
    const t = targets.find((value) => value.uri === packet.uri);
    assert.ok(t);
    assert.ok(packet.version === 1 || packet.version === 2);
    const expected = expectedDiagnostics(
      packet.version === 1 ? t.invalid : t.repair,
      kind,
      packet.version === 2,
    );
    const acceptable = [[], [pending], expected];
    assert.ok(
      acceptable.some((value) => isDeepStrictEqual(value, packet.diagnostics)),
      "unknown/stale whole diagnostic publication",
    );
  }
  for (const t of targets)
    for (const version of [1, 2]) {
      const rows = packets.filter((packet) => packet.uri === t.uri && packet.version === version);
      assert.ok(rows.length > 0);
      assert.deepEqual(rows.at(-1), {
        uri: t.uri,
        version,
        diagnostics: expectedDiagnostics(version === 1 ? t.invalid : t.repair, kind, version === 2),
      });
    }
}
export function inventory(root: string): Obj {
  let vueFiles = 0,
    vueAndTypeScriptFiles = 0,
    totalFiles = 0,
    totalBytes = 0;
  const visit = (directory: string): void => {
    for (const name of fs.readdirSync(directory)) {
      if (name === "node_modules" || name === ".git") continue;
      const file = path.join(directory, name);
      const stat = fs.lstatSync(file);
      if (stat.isSymbolicLink()) continue;
      if (stat.isDirectory()) visit(file);
      else if (stat.isFile()) {
        totalFiles++;
        totalBytes += stat.size;
        if (name.endsWith(".vue")) vueFiles++;
        if (/\.(?:vue|[cm]?tsx?)$/.test(name)) vueAndTypeScriptFiles++;
      }
    }
  };
  visit(root);
  return {
    vueFiles,
    vueAndTypeScriptFiles,
    totalFiles,
    totalBytes,
    scope: "filesystem names/stat only; no diagnostic scan or closed-file content hashing",
  };
}

export function diagnosticReceipts(receipts: Obj[], targets: Target[], kind: Fixture["id"]): Obj {
  const rows = targets.map((t) => {
    const expected = {
      uri: t.uri,
      version: 1,
      diagnostics: expectedDiagnostics(t.invalid, kind, false),
    };
    const receipt = receipts.find((row) => isDeepStrictEqual(row.params, expected));
    assert.ok(receipt, "actual exact native diagnostic receipt required");
    assert.match(String(receipt.ns), /^\d+$/);
    return { uri: t.uri, ...receipt, ns: String(receipt.ns) };
  });
  const ordered = rows.map((row) => row.ns).sort((a, b) => (BigInt(a) < BigInt(b) ? -1 : 1));
  return { rows, firstNs: ordered[0], allNs: ordered.at(-1) };
}
