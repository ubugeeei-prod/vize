import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type GlobalCase = {
  id: string;
  source: string;
  globals: string;
  oracle: string;
  discovery?: boolean;
  binding?: string;
  unusedDirective?: boolean;
  forbiddenReceiverNames?: string[];
  errors: Array<{ anchor: string; token: string; kind: string; suppressed?: boolean }>;
};
export type Packet = {
  label: string;
  command: string;
  args: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  elapsedMs: number;
};
export type Transform = {
  text: string;
  extension: string;
  mappings: number[][];
  diagnostics: unknown[];
  [key: string]: unknown;
};
export type GoldenPacket = {
  label: string;
  sourceSha256: string;
  globalsSha256: string;
  result: Transform;
};
export type PacketGoldens = {
  schema: string;
  version: number;
  issue: number;
  complete: boolean;
  packets: GoldenPacket[];
  [key: string]: unknown;
};
export type Response = { jsonrpc: string; id: number; result?: Transform; error?: unknown };
export const fixture = fileURLToPath(
  new URL("../../fixtures/typechecker/undeclared-instance-globals-10617/", import.meta.url),
);
export const sha = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

export function readGlobalsFixture() {
  const authority = JSON.parse(fs.readFileSync(path.join(fixture, "authority.json"), "utf8"));
  assert.equal(authority.schema, "vize.undeclared-instance-globals");
  assert.equal(authority.issue, 10617);
  assert.equal(authority.hostVersion, "7.1.0-beta");
  assert.equal(authority.publicVersion, "0.566.0");
  const files = new Map<string, string>();
  for (const input of authority.files) {
    assert.ok(!path.isAbsolute(input.path) && !input.path.split(/[\\/]/u).includes(".."));
    assert.equal(files.has(input.path), false);
    const bytes = fs.readFileSync(path.join(fixture, input.path));
    assert.equal(bytes.length, input.bytes, input.path);
    assert.equal(sha(bytes), input.sha256, input.path);
    files.set(input.path, bytes.toString("utf8"));
  }
  const lock = JSON.parse(files.get("project/package-lock.json.txt")!);
  assert.equal(lock.packages["node_modules/vize"].version, authority.publicVersion);
  assert.equal(lock.packages["node_modules/vize"].integrity, authority.publicIntegrity);
  assert.equal(lock.packages["node_modules/typescript"].integrity, authority.hostIntegrity);
  assert.equal(lock.packages["node_modules/vue"].version, authority.vueVersion);
  if (files.has("host-authority.json")) {
    const hostAuthority = JSON.parse(files.get("host-authority.json")!);
    const officialLock = JSON.parse(files.get("official-host/package-lock.json.txt")!);
    assert.equal(hostAuthority.schema, "vize.instance-globals-host-authority");
    assert.equal(hostAuthority.officialNightly.version, "7.1.0-dev.20261024.1");
    assert.equal(
      officialLock.packages["node_modules/typescript"].version,
      hostAuthority.officialNightly.version,
    );
    assert.equal(
      officialLock.packages["node_modules/typescript"].integrity,
      hostAuthority.officialNightly.integrity,
    );
    assert.equal(hostAuthority.sourceHost.revision, "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799");
  }
  const data = JSON.parse(files.get("cases.json")!) as {
    version: number;
    issue: number;
    cases: GlobalCase[];
  };
  assert.equal(data.issue, 10617);
  assert.equal(data.version, 1);
  return { authority, files, cases: data.cases };
}

export function frame(value: unknown) {
  const body = Buffer.from(JSON.stringify(value));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
}
export function decodeFrames(text: string): Response[] {
  const result: Response[] = [];
  let bytes = Buffer.from(text);
  while (bytes.length) {
    const end = bytes.indexOf("\r\n\r\n");
    assert.notEqual(end, -1);
    const header = /^Content-Length: (\d+)$/u.exec(bytes.subarray(0, end).toString());
    assert.ok(header);
    const start = end + 4,
      length = Number(header[1]);
    assert.ok(bytes.length >= start + length);
    result.push(JSON.parse(bytes.subarray(start, start + length).toString()));
    bytes = bytes.subarray(start + length);
  }
  return result;
}
export const request = (id: number, method: string, params: unknown) => ({
  jsonrpc: "2.0",
  id,
  method,
  params,
});

export function compilerRows(stdout: string) {
  const rows: Array<{ file: string; line: number; column: number; code: number; message: string }> =
    [];
  for (const line of stdout.trimEnd().split("\n")) {
    if (!line) continue;
    const match = /^(.+)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match)
      rows.push({
        file: match[1],
        line: Number(match[2]),
        column: Number(match[3]),
        code: Number(match[4]),
        message: match[5],
      });
    else {
      assert.ok(rows.length && /^\s/u.test(line), line);
      rows.at(-1)!.message += `\n${line}`;
    }
  }
  return rows;
}
export function location(source: string, anchor: string, token: string) {
  const start = source.indexOf(anchor);
  assert.notEqual(start, -1, anchor);
  assert.equal(source.indexOf(anchor, start + 1), -1, anchor);
  const offset = start + anchor.indexOf(token);
  const lines = source.slice(0, offset).split("\n");
  return {
    line: lines.length,
    column: lines.at(-1)!.length + 1,
    byteStart: Buffer.byteLength(source.slice(0, offset)),
    byteLength: Buffer.byteLength(token),
  };
}
export function expectedHost(
  source: string,
  entry: GlobalCase,
  rows: ReturnType<typeof compilerRows>,
) {
  assert.equal(rows.length, entry.errors.length, entry.id);
  const unused = entry.unusedDirective
    ? (() => {
        const at = location(source, "<!-- @vue-expect-error -->", "@vue-expect-error");
        return `src/Globals.vue(${at.line},${at.column}): error vize4: Unused '@vue-expect-error' directive\n`;
      })()
    : "";
  return (
    unused +
    rows
      .flatMap((row, index) => {
        const error = entry.errors[index];
        const at = location(source, error.anchor, error.token);
        return error.suppressed
          ? []
          : [`src/Globals.vue(${at.line},${at.column}): error TS${row.code}: ${row.message}\n`];
      })
      .join("")
  );
}
export function assertErrorMappings(source: string, entry: GlobalCase, result: Transform) {
  assert.equal(result.extension, ".tsx");
  assert.deepEqual(result.diagnostics, []);
  const generated = Buffer.from(result.text);
  for (const error of entry.errors.filter((item) => item.kind === "undeclared")) {
    const at = location(source, error.anchor, error.token);
    const links = result.mappings.filter(
      (span) => span[2] === at.byteStart && span[3] === at.byteLength,
    );
    assert.ok(links.length, `${entry.id}: exact authored UTF8 name mapping`);
    assert.ok(
      links.some(
        (span) => generated.subarray(span[0], span[0] + span[1]).toString() === error.token,
      ),
      entry.id,
    );
  }
}

export function assertWholeTransform(actual: Transform, expected: Transform) {
  assert.equal(
    JSON.stringify(actual),
    JSON.stringify(expected),
    "complete generated text, mapping order, directives and all opaque fields",
  );
}
export function assertTransformEnvelope(response: Response, id: number) {
  assert.ok(response.result, "a transform response must contain its complete result");
  assert.deepEqual(response, { id, jsonrpc: "2.0", result: response.result });
}
export function fullPacketMutations(golden: Transform, foreign: Transform) {
  const changes: Array<(copy: Transform) => void> = [
    (copy) => {
      copy.text += "\n// unexpected code";
    },
    (copy) => {
      copy.text = copy.text.replace("__ctx", "__bad");
    },
    (copy) => {
      copy.mappings.pop();
    },
    (copy) => {
      copy.mappings.push([...copy.mappings[0]]);
    },
    (copy) => {
      copy.extension = ".foreign";
    },
    (copy) => {
      copy.foreignField = true;
    },
    (copy) => {
      copy.diagnostics = [{ code: 9999, message: "foreign" }];
    },
    (copy) => {
      copy.diagnosticDirectives = { foreign: true };
    },
  ];
  for (const index of [0, 1, 2, 3, 4, 5])
    for (const delta of [-1, 1])
      changes.push((copy) => {
        copy.mappings[0][index] += delta;
      });
  if ("diagnosticDirectives" in golden) {
    changes.push((copy) => {
      delete copy.diagnosticDirectives;
    });
    changes.push((copy) => {
      const payload = copy.diagnosticDirectives as {
        unusedExpectDirectiveDiagnostics: Array<{ code: number }>;
      };
      payload.unusedExpectDirectiveDiagnostics[0].code = 9999;
    });
    for (const index of [0, 1, 2, 3, 4, 5])
      for (const delta of [-1, 1])
        changes.push((copy) => {
          const payload = copy.diagnosticDirectives as { directives: number[][] };
          payload.directives[0][index] += delta;
        });
  }
  for (const change of changes) {
    const copy = structuredClone(golden);
    change(copy);
    assert.notEqual(
      JSON.stringify(copy),
      JSON.stringify(golden),
      "negative control must actually change the packet",
    );
    assert.throws(() => assertWholeTransform(copy, golden));
  }
  assert.notEqual(JSON.stringify(foreign), JSON.stringify(golden));
  assert.throws(
    () => assertWholeTransform(foreign, golden),
    "a different authored case cannot acquire this input's packet",
  );
}

type AstNode = {
  end: number;
  text?: string;
  expression?: AstNode;
  name?: AstNode;
  getStart(source: AstNode): number;
};
type AstCompiler = {
  version: string;
  ScriptTarget: { Latest: number };
  ScriptKind: { TSX: number };
  createSourceFile(
    name: string,
    text: string,
    target: number,
    parents: boolean,
    kind: number,
  ): AstNode;
  isPropertyAccessExpression(node: AstNode): boolean;
  isIdentifier(node: AstNode): boolean;
  forEachChild(node: AstNode, visitor: (node: AstNode) => void): void;
};
export function instanceReceiverReads(result: Transform, referencePackage: string) {
  const ts = createRequire(referencePackage)("typescript") as AstCompiler;
  assert.equal(ts.version, "5.9.3");
  const parsed = ts.createSourceFile(
    "Globals.vue.tsx",
    result.text,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const reads: Array<{
    name: string;
    receiver: string;
    start: number;
    length: number;
    prefixStart: number;
    prefixLength: number;
  }> = [];
  const visit = (node: AstNode) => {
    if (
      ts.isPropertyAccessExpression(node) &&
      node.expression &&
      ts.isIdentifier(node.expression) &&
      ["__ctx", "__vize_strict_template_context"].includes(node.expression.text ?? "")
    ) {
      const name = node.name!;
      const start = Buffer.byteLength(result.text.slice(0, name.getStart(parsed)));
      const prefixStart = Buffer.byteLength(result.text.slice(0, node.expression.getStart(parsed)));
      reads.push({
        name: name.text!,
        receiver: node.expression.text!,
        prefixStart,
        prefixLength: start - prefixStart,
        start,
        length: Buffer.byteLength(result.text.slice(name.getStart(parsed), name.end)),
      });
    }
    ts.forEachChild(node, visit);
  };
  visit(parsed);
  return reads;
}

export function assertCheckedReadOwnership(
  source: string,
  entry: GlobalCase,
  result: Transform,
  referencePackage: string,
) {
  const reads = instanceReceiverReads(result, referencePackage);
  for (const name of entry.forbiddenReceiverNames ?? [])
    assert.equal(
      reads.filter((read) => read.name === name).length,
      0,
      `${entry.id}: an object/member key, literal, comment or callback-local name cannot acquire an instance receiver`,
    );
  for (const error of entry.errors.filter((item) => item.kind === "undeclared")) {
    const at = location(source, error.anchor, error.token);
    const named = reads.filter((read) => read.name === error.token);
    assert.equal(
      named.length,
      entry.errors.filter((item) => item.kind === "undeclared" && item.token === error.token)
        .length,
      entry.id,
    );
    assert.ok(
      named.some((read) =>
        result.mappings.some(
          (span) =>
            span[0] === read.start &&
            span[1] === read.length &&
            span[2] === at.byteStart &&
            span[3] === at.byteLength &&
            span[4] === 0 &&
            span[5] === 1048575,
        ),
      ),
      `${entry.id}: the checked receiver property, not the alias declaration, owns this authored name`,
    );
  }
  const lexical =
    entry.id === "setup-shadow"
      ? "$undeclared"
      : ["loop-shadow", "slot-shadow"].includes(entry.id)
        ? "$ghost"
        : entry.id === "options-own-method"
          ? "$local"
          : undefined;
  if (lexical)
    assert.equal(
      reads.filter((read) => read.name === lexical).length,
      0,
      `${entry.id}: lexical owner must not be rebound to the Vue instance`,
    );
}

export function assertProtocolAdmission(responses: Response[], configFile: string) {
  assert.deepEqual(responses[0], {
    id: 1,
    jsonrpc: "2.0",
    result: { diagnosticSource: "vize", positionEncoding: "utf-8" },
  });
  const watchedFiles = [fs.realpathSync(configFile)];
  assert.deepEqual(responses[1], {
    id: 2,
    jsonrpc: "2.0",
    result: {
      configIdentity: `vize-content-mapper-v1:${sha(JSON.stringify([{}, watchedFiles]))}`,
      watchedFiles,
    },
  });
}

export function writeInputs(root: string, files: Map<string, string>) {
  for (const [relative, bytes] of files) {
    if (!relative.startsWith("project/")) continue;
    const target = path.join(root, relative.slice("project/".length, -".txt".length));
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes);
  }
}
export function makeSourceProject(
  root: string,
  published: string,
  repoRoot: string,
  binary: string,
  files: Map<string, string>,
) {
  writeInputs(root, files);
  const modules = path.join(root, "node_modules");
  fs.mkdirSync(modules, { recursive: true });
  for (const name of fs.readdirSync(path.join(published, "node_modules"))) {
    if (name === "vize" || name === ".bin" || name === ".package-lock.json") continue;
    fs.symlinkSync(
      path.join(published, "node_modules", name),
      path.join(modules, name),
      process.platform === "win32" ? "junction" : "dir",
    );
  }
  const adapter = path.join(modules, "vize");
  fs.mkdirSync(adapter, { recursive: true });
  const manifest = JSON.parse(fs.readFileSync(path.join(repoRoot, "npm/cli/package.json"), "utf8"));
  manifest.private = true;
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  fs.writeFileSync(path.join(adapter, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  const helperFiles = [
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
    "generated/content-mapper-types.d.ts",
    "virtual-ts-globals.d.ts",
    "virtual-ts-patterns.d.ts",
    "virtual-ts-slots.d.ts",
    "virtual-ts.d.ts",
    "virtual-ts.d.mts",
    "virtual-ts.d.cts",
  ];
  for (const name of helperFiles) {
    const target = path.join(adapter, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(repoRoot, "npm/cli", name), target);
  }
  return {
    role: "source protocol adapter, not public-package acceptance",
    manifestSha256: sha(fs.readFileSync(path.join(adapter, "package.json"))),
    binarySha256: sha(fs.readFileSync(binary)),
    helperFiles: helperFiles.map((name) => ({
      name,
      sha256: sha(fs.readFileSync(path.join(adapter, name))),
    })),
  };
}
