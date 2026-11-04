import assert from "node:assert/strict";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  existsSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { decodeFrames, sha256, write } from "./template-alias-paired-11822-custody.mjs";

const order = (a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b));

export function directiveAuthority(packet, authoredBytes, virtualBytes) {
  if (packet === undefined || packet === null) return null;
  assert.ok(Array.isArray(packet.directives));
  assert.ok(Array.isArray(packet.unusedExpectDirectiveDiagnostics));
  const directives = packet.directives
    .map((tuple) => {
      assert.equal(tuple.length, 6);
      assert.ok(tuple.every((number) => Number.isSafeInteger(number) && number >= 0));
      const [start, length, generatedStart, generatedEnd, policy, unused] = tuple;
      assert.ok(start + length <= authoredBytes, "directive authored range escapes source");
      assert.ok(
        generatedStart <= generatedEnd && generatedEnd <= virtualBytes,
        "directive virtual range escapes transform",
      );
      assert.ok(policy === 0 || policy === 1);
      if (policy === 1) assert.ok(unused < packet.unusedExpectDirectiveDiagnostics.length);
      return [start, length, policy, unused];
    })
    .sort(order);
  return { directives, unusedExpectDirectiveDiagnostics: packet.unusedExpectDirectiveDiagnostics };
}

function packageManifest(from, name) {
  let current = from;
  for (;;) {
    const path = join(current, "node_modules", name, "package.json");
    if (existsSync(path)) return realpathSync(path);
    const parent = dirname(current);
    if (parent === current) throw new Error(`cannot resolve ${name} from ${from}`);
    current = parent;
  }
}

export function sharedHelperClosure(workspace) {
  const root = join(workspace, "node_modules/vize");
  const projectVue = packageManifest(workspace, "vue");
  const helperVue = packageManifest(root, "vue");
  assert.equal(helperVue, projectVue, "helper and authored code resolved different Vue instances");
  const declaration = readFileSync(join(root, "content-mapper-helpers.d.ts"), "utf8");
  assert.ok(
    declaration
      .split(/\r?\n/)
      .includes(
        'export type * from "./generated/content-mapper-types.js" with { "resolution-mode": "import" };',
      ),
  );
  const generated = readFileSync(join(root, "generated/content-mapper-types.d.ts"), "utf8");
  const exports = [...generated.matchAll(/export type\s*\{([^}]+)\}/g)].flatMap((match) =>
    match[1]
      .split(",")
      .map((name) => name.trim())
      .filter(Boolean),
  );
  assert.ok(exports.length > 0 && new Set(exports).size === exports.length);
  assert.ok(existsSync(join(root, "virtual-ts-checker.d.ts")));
  const virtual = readFileSync(join(root, "virtual-ts.d.mts"), "utf8");
  const virtualExports = [
    ...virtual.matchAll(
      /export\s+(?:declare\s+)?(?:type|interface|class|function|const)\s+([$\w]+)/g,
    ),
  ].map((match) => match[1]);
  assert.ok(virtualExports.length > 0);
  return {
    projectVue,
    helperVue,
    canonicalExports: exports.sort(),
    virtualExports: virtualExports.sort(),
    requiredMode: "shared-hoisted",
  };
}

export function admitSharedTransform(text, helpers) {
  assert.ok(
    text.includes('import type {} from "vize/virtual-ts-checker";'),
    "inline helper fallback is not an equivalent primary cohort",
  );
  const imports = [
    ...text.matchAll(/import type\s*\{([^}]*)\}\s*from\s*["']vize\/content-mapper-helpers["']/g),
  ].flatMap((match) =>
    match[1]
      .split(",")
      .map((specifier) => specifier.trim().split(/\s+as\s+/)[0])
      .filter(Boolean),
  );
  const canonical = imports.filter((name) => name.startsWith("$") || name.startsWith("__")).sort();
  assert.deepEqual(
    canonical,
    [],
    "shared-hoisted cohort unexpectedly contains the per-file canonical helper import block",
  );
  const references = [
    ...new Set(
      [...text.matchAll(/import\(["']vize\/virtual-ts["']\)\.([$\w]+)/g)].map((match) => match[1]),
    ),
  ].sort();
  assert.ok(references.length > 0, "shared SDK references absent");
  assert.ok(
    references.every((name) => helpers.virtualExports.includes(name)),
    "shared virtual import set contains a name absent from the actual SDK",
  );
  return {
    mode: "shared-hoisted",
    canonicalImports: canonical,
    virtualImports: references,
    otherHelperImports: imports.filter((name) => !canonical.includes(name)).sort(),
  };
}

export function collectTransforms(
  captureDirectory,
  workspace,
  output,
  expectedPaths,
  helpers = null,
) {
  mkdirSync(output);
  const input = new Map();
  const pending = new Set();
  const sourcePaths = [...expectedPaths].sort();
  assert.equal(new Set(sourcePaths).size, sourcePaths.length);
  for (const name of readdirSync(captureDirectory).filter((name) => name.startsWith("in-"))) {
    for (const message of decodeFrames(readFileSync(join(captureDirectory, name)))) {
      if (message.id === undefined) continue;
      const key = `${name.slice(3)}:${message.id}`;
      assert.equal(input.has(key), false, "duplicate protocol request id");
      input.set(key, message);
      if (message.method === "transform") pending.add(key);
    }
  }
  const transforms = [];
  const observed = new Set();
  for (const name of readdirSync(captureDirectory).filter((name) => name.startsWith("out-"))) {
    for (const message of decodeFrames(readFileSync(join(captureDirectory, name)))) {
      if (message.id === undefined) continue;
      const key = `${name.slice(4)}:${message.id}`;
      const request = input.get(key);
      assert.ok(request, "unmatched protocol response");
      if (request.method !== "transform") continue;
      assert.ok(pending.delete(key), "duplicate transform response");
      assert.equal(message.error, undefined, "failed transform response");
      assert.equal(typeof message.result?.text, "string", "transform missing or failed");
      const path = relative(workspace, request.params.fileName);
      assert.ok(!path.startsWith("..") && !isAbsolute(path), "transform escaped corpus");
      assert.ok(sourcePaths.includes(path), `unexpected authored transform ${path}`);
      assert.equal(observed.has(path), false, "duplicate authored transform");
      observed.add(path);
      const target = join(output, path + (message.result.extension ?? ".ts"));
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, message.result.text, { flag: "wx" });
      const { text, ...metadata } = message.result;
      write(target + ".metadata.json", { request, result: metadata });
      transforms.push({
        path,
        bytes: Buffer.byteLength(text),
        lines: text.split("\n").length,
        sha256: sha256(text),
        metadata,
        helperMode: helpers ? admitSharedTransform(text, helpers) : null,
        directiveAuthority: directiveAuthority(
          metadata.diagnosticDirectives,
          Buffer.byteLength(readFileSync(join(workspace, path))),
          Buffer.byteLength(text),
        ),
      });
    }
  }
  assert.equal(pending.size, 0, "unanswered transform requests");
  assert.deepEqual(
    [...observed].sort(),
    sourcePaths,
    "transforms do not match the exact authored Vue path set",
  );
  transforms.sort((a, b) => a.path.localeCompare(b.path));
  write(join(output, "inventory.json"), transforms);
  return {
    count: transforms.length,
    bytes: transforms.reduce((n, t) => n + t.bytes, 0),
    lines: transforms.reduce((n, t) => n + t.lines, 0),
    helperModes: transforms.map(({ path, helperMode }) => ({ path, helperMode })),
    directiveAuthority: transforms.map(({ path, directiveAuthority }) => ({
      path,
      directiveAuthority,
    })),
  };
}
