import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const navigationFixture = fileURLToPath(
  new URL("../../fixtures/typechecker/experimental-router-navigation-5.4/", import.meta.url),
);
export const productPage = "src/pages/products/[id=int].vue";

export function readNavigationFixture() {
  const authority = JSON.parse(
    fs.readFileSync(path.join(navigationFixture, "authority.json"), "utf8"),
  );
  assert.equal(authority.schema, "vize.router-published-navigation-matrix");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 10577);
  assert.equal(authority.provider.version, "5.4.0");
  assert.equal(authority.publicOracle.version, "0.562.0");
  assert.equal(
    authority.publicOracle.integrity,
    "sha512-yMbgpshr4M+7blGdarh9mHTpi1lcb5kbkyFIhe5s2Q9whmogyibi2IyOr+a6vCNvpSTE0ImgqnIrVX+aYR0LaA==",
  );
  const files = new Map<string, string>();
  for (const input of authority.inputs) {
    assert.equal(path.isAbsolute(input.path), false);
    assert.equal(input.path.split(/[\\/]/u).includes(".."), false);
    assert.equal(files.has(input.path), false, `duplicate fixture authority ${input.path}`);
    const bytes = fs.readFileSync(path.join(navigationFixture, input.path));
    assert.equal(bytes.length, input.bytes, input.path);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), input.sha256, input.path);
    files.set(input.path, bytes.toString("utf8"));
  }
  const generated = JSON.parse(files.get("generated-modules.json")!);
  for (const output of authority.generatedOutputs) {
    const source = generated[output.path] as string;
    assert.equal(Buffer.byteLength(source), output.bytes, output.path);
    assert.equal(createHash("sha256").update(source).digest("hex"), output.sha256, output.path);
    files.set(`${output.path}.txt`, source);
  }
  const provider = JSON.parse(files.get("provider.json")!);
  const providerLock = JSON.parse(files.get("provider-package-lock.json.txt")!);
  const checkerLock = JSON.parse(files.get("checker-host-package-lock.json.txt")!);
  const publicLock = JSON.parse(files.get("public-oracle-package-lock.json.txt")!);
  assert.deepEqual(provider.provider, {
    ...provider.provider,
    name: authority.provider.name,
    version: authority.provider.version,
    integrity: authority.provider.integrity,
  });
  assert.equal(
    providerLock.packages["node_modules/vue-router"].integrity,
    authority.provider.integrity,
  );
  assert.equal(
    checkerLock.packages["node_modules/typescript"].version,
    authority.checkerHost.version,
  );
  assert.equal(publicLock.packages["node_modules/vize"].version, "0.562.0");
  assert.equal(
    publicLock.packages["node_modules/vize"].integrity,
    authority.publicOracle.integrity,
  );
  return { authority, files };
}

export function authoredPages(files: Map<string, string>) {
  return new Map(
    [...files]
      .filter(([name]) => name.startsWith("project/") && name.endsWith(".vue.txt"))
      .map(([name, source]) => [name.slice("project/".length, -".txt".length), source]),
  );
}

export type MatrixCase = {
  id: string;
  file: string;
  source: string;
  anchor: string;
  token: string;
  code: number;
  link?: boolean;
};

export function invalidMatrixCases(pages: Map<string, string>): MatrixCase[] {
  const product = pages.get(productPage)!;
  const cases: MatrixCase[] = [];
  for (const [id, before, after, token] of [
    ["required-path", "const id: number", "const id: string", "id"],
    ["required-query", "const q: string", "const q: number", "q"],
    ["default-query", "const page: number", "const page: string", "page"],
    ["array-query", "const tags: string[]", "const tags: number[]", "tags"],
    ["custom-date-query", "const at: Date", "const at: string", "at"],
    ["hash", "const heading: string", "const heading: number", "heading"],
  ]) {
    assert.ok(product.includes(before), id);
    cases.push({
      id,
      file: productPage,
      source: product.replace(before, after),
      anchor: after,
      token,
      code: 2322,
    });
  }
  for (const [id, file, before] of [
    ["optional-path", "src/pages/optional/[[id=int]].vue", "const value: number | null"],
    ["repeated-path", "src/pages/repeat/[ids=int]+.vue", "const value: number[]"],
    [
      "optional-repeated-path",
      "src/pages/optional-repeat/[[ids=int]]+.vue",
      "const value: number[]",
    ],
  ]) {
    const source = pages.get(file)!;
    assert.ok(source.includes(before), id);
    cases.push({
      id,
      file,
      source: source.replace(before, "const value: string"),
      anchor: "const value: string",
      token: "value",
      code: 2322,
    });
  }
  for (const method of ["push", "replace", "resolve"]) {
    const anchor = `router.${method}({ name: '/products/[id=int]', params: { id: 1`;
    assert.ok(product.includes(anchor), method);
    const invalid = anchor.replace("id: 1", "id: 'wrong'");
    cases.push({
      id: `navigation-${method}`,
      file: productPage,
      source: product.replace(anchor, invalid),
      anchor: invalid,
      token: "{",
      code: method === "resolve" ? 2769 : 2345,
    });
  }
  const link = "<RouterLink :to=\"{ name: '/products/[id=int]', params: { id: 1";
  const invalidLink = link.replace("id: 1", "id: 'wrong'");
  cases.push({
    id: "router-link-target",
    file: productPage,
    source: product.replace(link, invalidLink),
    anchor: invalidLink,
    token: "to",
    code: 2322,
    link: true,
  });
  return cases;
}

export function point(source: string, anchor: string, token: string) {
  const start = source.indexOf(anchor);
  assert.notEqual(start, -1, anchor);
  assert.equal(source.indexOf(anchor, start + 1), -1, `unique anchor ${anchor}`);
  const prefix = source.slice(0, start + anchor.indexOf(token));
  const lines = prefix.split("\n");
  return { line: lines.length, column: lines.at(-1)!.length + 1 };
}

/** Independent TS program: public route types, no generated Vize source. */
export function plainTypeScriptOracle(source: string, file: string, link = false) {
  const routeName = `/${file.slice("src/pages/".length, -".vue".length)}`;
  const script = source
    .slice(source.indexOf("\n") + 1, source.indexOf("</script>"))
    .replace("import { definePage } from 'vue-router/experimental'\n", "")
    .replace(/definePage\(\{[\s\S]*?^\}\)\r?\n/mu, "")
    .replace("useRoute()", `useRoute(${JSON.stringify(routeName)})`);
  if (!link) return script;
  const expression = source.match(/<RouterLink :to="([^"]+)"/u)?.[1];
  assert.ok(expression);
  return `${script}\nconst linkTarget: import('vue-router').RouterLinkProps['to'] = ${expression}\n`;
}

export function parseCompilerDiagnostics(stdout: string) {
  const rows: Array<{ file: string; line: number; column: number; code: number; message: string }> =
    [];
  for (const line of stdout.trimEnd().split("\n")) {
    if (!line) continue;
    const match = line.match(/^(.+)\((\d+),(\d+)\): error TS(\d+): (.*)$/u);
    if (match) {
      rows.push({
        file: match[1],
        line: Number(match[2]),
        column: Number(match[3]),
        code: Number(match[4]),
        message: match[5],
      });
    } else {
      assert.ok(rows.length && /^\s/u.test(line), `unclassified compiler output: ${line}`);
      rows.at(-1)!.message += `\n${line.trimStart()}`;
    }
  }
  return rows;
}

/** Each probe states which authored surface the independent reference admits. */
export function formatProbes() {
  const script = `import { useRoute } from 'vue-router'\nconst route = useRoute('/products/[id=int]')\nroute.params.id.toUpperCase()\n`;
  return [
    {
      id: "vue-js",
      file: "src/format-probe.vue",
      source: `<script setup>\n${script}</script>\n<template><div>{{ route.params.id }}</div></template>\n`,
      reference: "vue-tsc",
      limitation: null,
    },
    {
      id: "ts",
      file: "src/format-probe.ts",
      source: script,
      reference: "typescript",
      limitation: "explicit named route; no file-inferred page identity",
    },
    {
      id: "tsx",
      file: "src/format-probe.tsx",
      source: `${script}const view = <div>{route.params.id}</div>\n`,
      reference: "typescript",
      limitation: "explicit named route; no file-inferred page identity",
    },
    {
      id: "art",
      file: "src/format-probe.art.vue",
      source: `<script setup lang="ts">\n${script}</script>\n<art><variant name="Route"><div>{{ route.params.id.toUpperCase() }}</div></variant></art>\n`,
      reference: "vue-tsc",
      limitation:
        "the independent reference admits script, but has no art variant-template adapter",
    },
    ...["pug", "jade"].map((language) => ({
      id: language,
      file: "src/format-probe.vue",
      source: `<script setup lang="ts">\n${script}</script>\n<template lang="${language}">\ndiv {{ route.params.id.toUpperCase() }}\n</template>\n`,
      reference: "vue-tsc",
      limitation: `the pinned independent reference has no ${language} template adapter; script diagnostics remain meaningful`,
    })),
  ];
}

export type FormatStage = {
  stage: string;
  nativeStatus: number | null;
  referenceStatus: number | null;
  nativeReport: { files?: Array<{ file: string; diagnostics: string[] }> } | null;
  referenceDiagnostics: string;
  referenceStderr: string;
};

export function classifyFormatProbe(file: string, stages: FormatStage[]) {
  const invalid = stages.find(({ stage }) => stage === "invalid")!;
  const repaired = stages.find(({ stage }) => stage === "repaired")!;
  let reference: ReturnType<typeof parseCompilerDiagnostics>;
  try {
    reference = parseCompilerDiagnostics(invalid.referenceDiagnostics);
  } catch (error) {
    return {
      script: "refused",
      reason: `independent output is not a compiler diagnostic packet: ${String(error)}`,
    };
  }
  const meaningful =
    invalid.referenceStatus === 2 &&
    reference.length === 1 &&
    reference[0].file === file &&
    reference[0].code === 2339 &&
    repaired.referenceStatus === 0 &&
    repaired.referenceDiagnostics === "";
  if (!meaningful)
    return {
      script: "unverified",
      reason:
        "the independent reference did not establish exactly one invalid script diagnostic and a clean repair",
    };
  const nativeInvalid = invalid.nativeReport?.files?.find((row) => row.file === file);
  const nativeRepaired = repaired.nativeReport?.files?.find((row) => row.file === file);
  if (!nativeInvalid || !nativeRepaired)
    return {
      script: "not-admitted",
      reason: "the native report did not admit the authored probe file",
    };
  const expected = `error:${reference[0].line}:${reference[0].column} [TS${reference[0].code}] ${reference[0].message}`;
  const pass =
    invalid.nativeStatus === 1 &&
    !invalid.nativeReport!.files!.some((row) => row.file !== file && row.diagnostics.length) &&
    nativeInvalid.diagnostics.includes(expected) &&
    repaired.nativeStatus === 0 &&
    nativeRepaired.diagnostics.length === 0 &&
    invalid.referenceStderr === "" &&
    repaired.referenceStderr === "";
  return {
    script: pass ? "pass" : "gap",
    reason: pass
      ? "complete independent script diagnostic and clean repair agree"
      : "native admission, complete script diagnostic or clean repair differs from the independent reference",
    extraNativeDiagnostics: nativeInvalid.diagnostics.filter(
      (diagnostic) => diagnostic !== expected,
    ),
  };
}
