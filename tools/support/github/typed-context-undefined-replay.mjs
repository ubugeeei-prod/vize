import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  readFileSync,
  writeFileSync,
  appendFileSync,
  readdirSync,
  realpathSync,
  mkdirSync,
  cpSync,
  copyFileSync,
} from "node:fs";
import { resolve, join, basename } from "node:path";
const [mode, outArg, phase] = process.argv.slice(2),
  out = resolve(outArg);
const hash = (b) => createHash("sha256").update(b).digest("hex"),
  json = (p, v) => writeFileSync(p, JSON.stringify(v, null, 2) + "\n");
const family = "tests/_fixtures/differential/typechecker/template-context-undefined-10667";
const observer = "crates/vize/tests/content_mapper_tsgo_cli/typed_context_undefined.rs";
const sourceSha = readFileSync(join(out, "source.sha"), "utf8").trim();
const embeddedHelperPaths = [
  "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/prop-keys.d.ts.txt",
  null,
  "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/native-elements.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/component-global-attributes.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/for-list.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/directives.d.ts.txt",
];
const embeddedHelpers = (root) =>
  Buffer.concat(
    embeddedHelperPaths.map((path) =>
      path === null ? Buffer.from("\n") : readFileSync(join(root, path)),
    ),
  );
const overlap = (start, length, edit) => start < edit.end && edit.start < start + length;
const point = (value, edits) => {
  assert(Number.isSafeInteger(value) && value >= 0);
  let delta = 0;
  for (const edit of edits) {
    if (value >= edit.end) delta += edit.replacement.length - (edit.end - edit.start);
    else {
      assert(value <= edit.start, "coordinate enters a certified generated replacement");
      break;
    }
  }
  return value + delta;
};
const projectedRange = (start, length, edits) => {
  assert(edits.every((edit) => !overlap(start, length, edit)));
  const relocated = point(start, edits);
  return [relocated, point(start + length, edits) - relocated];
};
const uniqueOffset = (text, needle) => {
  const offset = text.indexOf(needle);
  assert(offset >= 0 && text.indexOf(needle, offset + 1) < 0, "certified bytes must occur once");
  return offset;
};
const applyEdits = (text, edits) => {
  let cursor = 0;
  const fragments = [];
  for (const edit of edits) {
    assert(cursor <= edit.start && edit.start <= edit.end);
    fragments.push(text.subarray(cursor, edit.start), edit.replacement);
    cursor = edit.end;
  }
  fragments.push(text.subarray(cursor));
  return Buffer.concat(fragments);
};
const certifyMapperTransition = (beforePacket, afterPacket) => {
  const before = Buffer.from(beforePacket.text);
  const after = Buffer.from(afterPacket.text);
  const beforeHelper = embeddedHelpers("before");
  const afterHelper = embeddedHelpers("source");
  const helperStart = uniqueOffset(before, beforeHelper);
  uniqueOffset(after, afterHelper);
  const mapped = beforePacket.mappings.map(([start, length]) => [start, length]);
  const generated = (start, length) =>
    !mapped.some(([g, l]) => overlap(start, length, { start: g, end: g + l }));
  const primary = [
    {
      start: helperStart,
      end: helperStart + beforeHelper.length,
      replacement: afterHelper,
      label: "embedded helper block",
    },
  ];
  const beforeText = before.toString();
  for (const match of beforeText.matchAll(/\b__Vize[A-Za-z0-9_]+\b/gu)) {
    const start = Buffer.byteLength(beforeText.slice(0, match.index));
    const end = start + Buffer.byteLength(match[0]);
    if (helperStart <= start && end <= helperStart + beforeHelper.length) continue;
    if (!generated(start, end - start)) continue;
    primary.push({
      start,
      end,
      replacement: Buffer.from(`$${match[0].slice("__Vize".length)}`),
      label: "collision-safe generated identifier",
    });
  }
  for (const context of ["const defineModel = undefined", "const __ctx = undefined"]) {
    const at = uniqueOffset(before, Buffer.from(context));
    const start = at + Buffer.byteLength(context) - Buffer.byteLength("undefined");
    primary.push({
      start,
      end: start + Buffer.byteLength("undefined"),
      replacement: Buffer.from("(void 0)"),
      label: "undefined-safe generated value",
    });
  }
  const markerPrefix = "readonly __vizeRawProps?: Props; ";
  const markerNeedle = `${markerPrefix}readonly __vizeHasFallthroughProps: true;`;
  const markerStart =
    uniqueOffset(before, Buffer.from(markerNeedle)) + Buffer.byteLength(markerPrefix);
  primary.push({
    start: markerStart,
    end: markerStart,
    replacement: Buffer.from("readonly __vizeNativeFallthroughRoot: true; "),
    label: "native fallthrough ownership marker",
  });
  primary.sort((left, right) => left.start - right.start || left.end - right.end);
  for (let index = 1; index < primary.length; index++)
    assert(primary[index - 1].end <= primary[index].start, "certified generated edits overlap");
  const comments = [...beforeText.matchAll(/\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)/gu)];
  const commentEdits = comments.map((match) => {
    const start = Buffer.byteLength(beforeText.slice(0, match.index));
    const [generatedStart, generatedEnd, sourceStart, sourceEnd] = match.slice(1).map(Number);
    const replacement = Buffer.from(
      `// @vize-map: ${point(generatedStart, primary)}:${point(generatedEnd, primary)} -> ${sourceStart}:${sourceEnd}`,
    );
    return {
      start,
      end: start + Buffer.byteLength(match[0]),
      replacement,
      label: "derived map coordinates",
    };
  });
  const edits = [...primary, ...commentEdits].sort(
    (left, right) => left.start - right.start || left.end - right.end,
  );
  for (let index = 1; index < edits.length; index++)
    assert(edits[index - 1].end <= edits[index].start, "complete certified edits overlap");
  assert.deepEqual(after, applyEdits(before, edits), "all other generated bytes must remain exact");
  return {
    edits,
    beforeHelperSha256: hash(beforeHelper),
    afterHelperSha256: hash(afterHelper),
    generatedIdentifierEdits: primary.filter(
      (edit) => edit.label === "collision-safe generated identifier",
    ).length,
  };
};
const certifyNativeTransition = (beforeText, afterText) => {
  const before = Buffer.from(beforeText);
  const after = Buffer.from(afterText);
  const source = before.toString();
  const primary = [];
  for (const match of source.matchAll(/\b__Vize[A-Za-z0-9_]+\b/gu)) {
    const start = Buffer.byteLength(source.slice(0, match.index));
    primary.push({
      start,
      end: start + Buffer.byteLength(match[0]),
      replacement: Buffer.from(`$${match[0].slice("__Vize".length)}`),
      label: "collision-safe generated identifier",
    });
  }
  const context = "const __ctx = undefined";
  const contextAt = uniqueOffset(before, Buffer.from(context));
  const contextStart = contextAt + Buffer.byteLength(context) - Buffer.byteLength("undefined");
  primary.push({
    start: contextStart,
    end: contextStart + Buffer.byteLength("undefined"),
    replacement: Buffer.from("(void 0)"),
    label: "undefined-safe generated value",
  });
  const markerPrefix = "readonly __vizeRawProps?: Props; ";
  const markerNeedle = `${markerPrefix}readonly __vizeHasFallthroughProps: true;`;
  const markerStart =
    uniqueOffset(before, Buffer.from(markerNeedle)) + Buffer.byteLength(markerPrefix);
  primary.push({
    start: markerStart,
    end: markerStart,
    replacement: Buffer.from("readonly __vizeNativeFallthroughRoot: true; "),
    label: "native fallthrough ownership marker",
  });
  primary.sort((left, right) => left.start - right.start || left.end - right.end);
  for (let index = 1; index < primary.length; index++)
    assert(primary[index - 1].end <= primary[index].start, "certified native edits overlap");
  const commentEdits = [...source.matchAll(/\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)/gu)].map(
    (match) => {
      const start = Buffer.byteLength(source.slice(0, match.index));
      const [generatedStart, generatedEnd, sourceStart, sourceEnd] = match.slice(1).map(Number);
      return {
        start,
        end: start + Buffer.byteLength(match[0]),
        replacement: Buffer.from(
          `// @vize-map: ${point(generatedStart, primary)}:${point(generatedEnd, primary)} -> ${sourceStart}:${sourceEnd}`,
        ),
        label: "derived map coordinates",
      };
    },
  );
  const edits = [...primary, ...commentEdits].sort(
    (left, right) => left.start - right.start || left.end - right.end,
  );
  for (let index = 1; index < edits.length; index++)
    assert(edits[index - 1].end <= edits[index].start, "complete certified native edits overlap");
  assert.deepEqual(
    after,
    applyEdits(before, edits),
    "all other native generated bytes must remain exact",
  );
  return {
    edits,
    generatedIdentifierEdits: primary.filter(
      (edit) => edit.label === "collision-safe generated identifier",
    ).length,
  };
};
if (mode === "prepare") {
  assert.equal(process.version, "v26.12.1");
  assert.match(sourceSha, /^[a-f0-9]{40}$/);
  const physical = (root, version) =>
    realpathSync(
      join(
        root,
        "node_modules/.pnpm",
        `@typescript+typescript-linux-x64@${version}`,
        "node_modules/@typescript/typescript-linux-x64/lib/tsc",
      ),
    );
  const native = physical("source", "7.0.2"),
    beta = physical("public", "7.1.0-beta"),
    vue = realpathSync("public/node_modules/vue");
  const providers = [];
  for (const [name, path, version] of [
    ["native-702", native, "7.0.2"],
    ["beta", beta, "7.1.0-beta"],
  ]) {
    const r = spawnSync(path, ["--version"]);
    assert.equal(r.status, 0);
    assert.equal(r.stdout.toString(), `Version ${version}\n`);
    assert.equal(r.stderr.length, 0);
    providers.push({
      name,
      path,
      version,
      sha256: hash(readFileSync(path)),
      stdout: [...r.stdout],
      stderr: [...r.stderr],
    });
  }
  assert.equal(
    providers[0].sha256,
    "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  );
  const pkg = JSON.parse(readFileSync("public/node_modules/vize/package.json", "utf8"));
  assert.equal(pkg.version, "0.569.0");
  mkdirSync("public/src", { recursive: true });
  copyFileSync(join("source", family, "sources/prop-used.vue.txt"), "public/src/Route.vue");
  const config = JSON.parse(readFileSync(join("source", family, "tsconfig.json.txt"), "utf8"));
  config.contentMappers = [{ package: "vize", extensions: [".vue"] }];
  json("public/tsconfig.json", config);
  const args = ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
    before = spawnSync(beta, args, { cwd: resolve("public") });
  writeFileSync(join(out, "public-before.stdout"), before.stdout);
  writeFileSync(join(out, "public-before.stderr"), before.stderr);
  const message = (code) =>
    `src/Route.vue(129,19): error TS${code}: ${code === 2448 ? "Block-scoped variable 'undefined' used before its declaration." : "Variable 'undefined' is used before being assigned."}\n  This location is in virtual code produced by the content mapper 'vize@0.569.0' and has no corresponding location in the original file.\n`;
  assert.equal(before.status, 2);
  assert.equal(before.stderr.length, 0);
  assert.equal(before.stdout.toString(), message(2448) + message(2454));
  json(join(out, "public-before.json"), {
    argv: [beta, ...args],
    status: before.status,
    stdout: [...before.stdout],
    stderr: [...before.stderr],
    source: readFileSync("public/src/Route.vue", "utf8"),
  });
  json(join(out, "provider-custody.json"), {
    source: sourceSha,
    node: process.version,
    providers,
    vue: { path: vue, manifestSha256: hash(readFileSync(join(vue, "package.json"))) },
    publishedPackage: {
      manifestSha256: hash(readFileSync("public/node_modules/vize/package.json")),
      version: pkg.version,
      lockSha256: hash(readFileSync("public/pnpm-lock.yaml")),
      projectPolicySha256: hash(readFileSync("public/pnpm-workspace.yaml")),
    },
  });
  appendFileSync(
    process.env.GITHUB_ENV,
    `VIZE_TEST_CONTEXT_UNDEFINED_NATIVE=${native}\nVIZE_TEST_CONTENT_MAPPER_TSGO=${beta}\nVIZE_TEST_CONTENT_MAPPER_VUE=${vue}\nCORSA_PATH=${native}\n`,
  );
  // The before image receives identical tests/data only; production files remain exact.
  cpSync(join("source", observer), join("before", observer));
  cpSync(join("source", observer.slice(0, -3)), join("before", observer.slice(0, -3)), {
    recursive: true,
  });
  cpSync(join("source", family), join("before", family), { recursive: true });
  appendFileSync(
    "before/crates/vize/tests/content_mapper_tsgo_cli.rs",
    "\nmod typed_context_undefined;\n",
  );
  const changed = spawnSync("git", ["diff", "--name-only"], { cwd: "before", encoding: "utf8" });
  assert.equal(changed.status, 0);
  assert.equal(changed.stdout, "crates/vize/tests/content_mapper_tsgo_cli.rs\n");
  json(join(out, "before-overlay.json"), {
    source: readFileSync(join(out, "before.sha"), "utf8").trim(),
    productionMutation: false,
    testOnlyTrackedChange: changed.stdout.trim(),
    observerSha256: hash(readFileSync(join("before", observer))),
    headObserverSha256: hash(readFileSync(join("source", observer))),
  });
} else if (mode === "seal") {
  const directory = join(out, phase);
  mkdirSync(directory, { recursive: true });
  const build = readFileSync(join(out, `${phase}-build.jsonl`), "utf8")
    .split("\n")
    .filter(Boolean)
    .map((s) => JSON.parse(s));
  const executables = build.filter((x) => x.reason === "compiler-artifact" && x.executable);
  assert(executables.length >= 2);
  const images = [];
  const checkout = resolve(phase === "before" ? "before" : "source");
  const target = resolve("target", phase === "before" ? "context-before" : "context-head");
  const prior =
    phase === "after" ? JSON.parse(readFileSync(join(out, "before-images.json"), "utf8")) : null;
  for (const row of executables) {
    assert(
      row.package_id.startsWith(`path+file://${checkout}/`),
      "compiler artifact belongs to the actual source checkout",
    );
    assert(
      row.executable.startsWith(`${target}/`),
      "before and head executable images have isolated build ownership",
    );
    const data = readFileSync(row.executable),
      destination = join(directory, basename(row.executable));
    copyFileSync(row.executable, destination);
    const previous = prior?.images.find((image) => image.target.name === row.target.name);
    if (previous)
      assert.notEqual(
        hash(data),
        previous.sha256,
        "head image must not reuse the before executable",
      );
    images.push({
      path: row.executable,
      retained: destination,
      sha256: hash(data),
      packageId: row.package_id,
      target: row.target,
    });
  }
  json(join(out, `${phase}-images.json`), {
    source: readFileSync(
      join(out, phase === "before" ? "before.sha" : "source.sha"),
      "utf8",
    ).trim(),
    images,
  });
} else if (mode === "verify") {
  const packets = (root) => {
    const all = [];
    function walk(p) {
      for (const e of readdirSync(p, { withFileTypes: true })) {
        const f = join(p, e.name);
        if (e.isDirectory()) walk(f);
        else if (e.isFile() && f.endsWith(".json")) {
          const d = JSON.parse(readFileSync(f, "utf8"));
          if (
            d.schema === "vize.typed-context-undefined-host" ||
            d.schema === "vize.typed-context-options-host"
          ) {
            assert.deepEqual(readFileSync(f.slice(0, -5) + ".stdout.raw"), Buffer.from(d.stdout));
            assert.deepEqual(readFileSync(f.slice(0, -5) + ".stderr.raw"), Buffer.from(d.stderr));
            all.push({ path: f, value: d });
          }
        }
      }
    }
    walk(root);
    return all;
  };
  const before = packets("before/target/vize-tests/typed-context-undefined-10667"),
    after = packets("source/target/vize-tests/typed-context-undefined-10667"),
    options = packets("source/target/vize-tests/typed-context-undefined-options-10667");
  assert.equal(readFileSync(join(out, "before.status"), "utf8"), "101\n");
  assert(readFileSync(join(out, "before.stderr"), "utf8").includes("lf/prop-used/native-raw-702"));
  assert.equal(before.length, 240);
  assert.equal(readFileSync(join(out, "ast.status"), "utf8"), "0\n");
  assert.equal(readFileSync(join(out, "after.status"), "utf8"), "0\n");
  assert.equal(readFileSync(join(out, "options.status"), "utf8"), "0\n");
  assert.equal(after.length, 240);
  assert.equal(options.length, 48);
  const minimal = before.filter((p) => p.path.includes("/lf/prop-used/"));
  assert.equal(minimal.length, 4);
  const baseline = Object.fromEntries(minimal.map((p) => [p.value.host, p.value]));
  for (const host of ["native-raw-702", "native-raw-beta", "mapper-beta"]) {
    const p = baseline[host];
    assert.equal(p.stderr.length, 0);
    assert.equal(p.status, host === "native-raw-702" ? 1 : 2);
    const text = host === "mapper-beta" ? p.inputs.mapperTransform.text : p.inputs.virtualCode;
    const marker = text.indexOf("const __ctx = undefined") + "const __ctx = ".length;
    assert(marker > 20);
    const prefix = text.slice(0, marker),
      line = prefix.split("\n").length,
      column = prefix.slice(prefix.lastIndexOf("\n") + 1).length + 1;
    const file = host === "mapper-beta" ? "src/Route.vue" : "src/Route.vue.ts";
    const expected = [2448, 2454]
      .map(
        (code) =>
          `${file}(${line},${column}): error TS${code}: ${code === 2448 ? "Block-scoped variable 'undefined' used before its declaration." : "Variable 'undefined' is used before being assigned."}\n${host === "mapper-beta" ? "  This location is in virtual code produced by the content mapper 'vize' and has no corresponding location in the original file.\n" : ""}`,
      )
      .join("");
    assert.equal(Buffer.from(p.stdout).toString(), expected);
  }
  assert.equal(baseline.cli.status, 0);
  assert.equal(JSON.parse(Buffer.from(baseline.cli.stdout).toString()).errorCount, 0);
  // Same compiled source packets: compare whole outputs and every unmodified map field.
  const post = after.find((p) => p.path.includes("/lf/prop-used/native-raw-702.json")).value;
  const pre = baseline["native-raw-702"].inputs;
  assert.equal(post.inputs.source, pre.source);
  const nativeTransition = certifyNativeTransition(pre.virtualCode, post.inputs.virtualCode);
  const a = pre.mapperTransform,
    b = post.inputs.mapperTransform;
  const transition = certifyMapperTransition(a, b);
  const edits = transition.edits;
  assert.equal(a.extension, b.extension);
  assert.deepEqual(a.diagnostics, b.diagnostics);
  assert.deepEqual(a.diagnosticDirectives, b.diagnosticDirectives);
  const mappings = a.mappings.map(([start, length, ...rest]) => {
    return [...projectedRange(start, length, edits), ...rest];
  });
  assert.deepEqual(b.mappings, mappings);
  const links = a.semanticLinks ?? [];
  assert.deepEqual(
    b.semanticLinks ?? [],
    links.map((l) => ({
      ...l,
      sourceStart: projectedRange(l.sourceStart, l.sourceLength, edits)[0],
      sourceLength: projectedRange(l.sourceStart, l.sourceLength, edits)[1],
      targetStart: projectedRange(l.targetStart, l.targetLength, edits)[0],
      targetLength: projectedRange(l.targetStart, l.targetLength, edits)[1],
    })),
  );
  for (const p of [...before, ...after]) {
    assert.equal(
      p.value.inputs.identity.sourceRevision,
      p.path.startsWith("before/")
        ? readFileSync(join(out, "before.sha"), "utf8").trim()
        : sourceSha,
    );
    assert(p.value.inputs.identity.images.every((i) => /^[a-f0-9]{64}$/.test(i.sha256)));
  }
  json(join(out, "verification.json"), {
    source: sourceSha,
    beforeSource: readFileSync(join(out, "before.sha"), "utf8").trim(),
    beforePackets: before.length,
    afterPackets: after.length,
    optionPackets: options.length,
    unfilteredRawOutcomes: true,
    oldCliFalseCleanPreserved: true,
    wholeNativeCodeOnlyCertifiedGeneratedContractsAndContextLiteralChanged: true,
    nativeCollisionSafeGeneratedIdentifierEdits: nativeTransition.generatedIdentifierEdits,
    wholeMapperCodeOnlyCertifiedGeneratedContractsAndContextLiteralsChanged: true,
    beforeEmbeddedHelperSha256: transition.beforeHelperSha256,
    afterEmbeddedHelperSha256: transition.afterHelperSha256,
    collisionSafeGeneratedIdentifierEdits: transition.generatedIdentifierEdits,
    allAuthoredRangesAndFeaturesExact: true,
    productionMappingNormalization: false,
    packets: [...before, ...after, ...options].map((p) => ({
      path: p.path,
      sha256: hash(readFileSync(p.path)),
      host: p.value.host,
      status: p.value.status,
      stdoutBytes: p.value.stdout.length,
      stderrBytes: p.value.stderr.length,
    })),
  });
} else throw Error("unknown replay mode");
