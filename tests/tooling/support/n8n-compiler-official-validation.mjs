// Assertions dereference graph pointers without modifying any retained official packet.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { read, sha256 } from "./n8n-compiler-custody-receipt.mjs";

const compilerIdentity = {
  package: "@vue/compiler-sfc",
  version: "3.5.26",
  entrypoint: "dist/compiler-sfc.esm-browser.js",
  url: "https://unpkg.com/@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js",
  sha256: "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec",
};
const licensePins = {
  "LICENSE.md": "d2f621f59aa4c10eab79b6333e59d9d3d5b53307dcfd16dbd75e40e679e84965",
  "LICENSE_EE.md": "6110c69fd3b92928328a89e863dc2c862ea449f2335c352a9c4a137a36b13040",
};

function graph(value) {
  const seen = new WeakMap();
  function visit(node) {
    if (!node || typeof node !== "object") return node;
    if (Object.hasOwn(node, "$ref")) {
      assert.deepEqual(Object.keys(node), ["$ref"]);
      assert.ok(node.$ref.startsWith("#/"));
      const target = node.$ref
        .slice(2)
        .split("/")
        .reduce((item, key) => {
          const unescaped = key.replaceAll("~1", "/").replaceAll("~0", "~");
          assert.ok(item && Object.hasOwn(item, unescaped), `missing pointer: ${node.$ref}`);
          return item[unescaped];
        }, value);
      return visit(target);
    }
    if (seen.has(node)) return seen.get(node);
    const result = Array.isArray(node) ? [] : {};
    seen.set(node, result);
    for (const [key, item] of Object.entries(node)) result[key] = visit(item);
    return result;
  }
  return visit(value);
}

const plain = (value) =>
  value === undefined
    ? undefined
    : JSON.parse(
        JSON.stringify(value, (_key, item) => {
          if (item instanceof Map) return { $map: [...item] };
          if (item instanceof Set) return { $set: [...item] };
          if (typeof item === "bigint") return { $bigint: String(item) };
          return item;
        }),
      );
const diagnostic = (error) => ({
  name: error.name,
  code: error.code,
  message: error.message,
  loc: plain(error.loc),
});
const modes = [
  { name: "function-no-prefix", mode: "function", prefixIdentifiers: false },
  { name: "module-prefix", mode: "module", prefixIdentifiers: true },
];
const options = (mode) => ({
  mode: mode.mode,
  prefixIdentifiers: mode.prefixIdentifiers,
  cacheHandlers: false,
  hoistStatic: true,
  comments: false,
});

function instanceErrors(sourceMap) {
  const starts = sourceMap
    ? [
        [710, 27497],
        [1246, 47563],
        [1246, 47563],
      ]
    : [
        [183, 7768],
        [192, 8105],
        [192, 8105],
      ];
  return starts.map(([line, offset]) => ({
    name: "SyntaxError",
    code: 29,
    message: "v-if/else branches must use unique keys.",
    loc: {
      start: { column: 5, line, offset },
      end: { column: 68, line, offset: offset + 63 },
      source: `:key="'floating-' + chunk.item.toolCall.confirmation.requestId"`,
    },
  }));
}

function attempt(callback) {
  const warnings = [];
  const original = console.warn;
  console.warn = (...args) => warnings.push(args.map(String).join(" "));
  try {
    return { result: callback(), warnings };
  } finally {
    console.warn = original;
  }
}

export async function validateOfficial(root, fixture, manifest) {
  const rawSummary = read(root, "capture.json");
  const summary = graph(JSON.parse(rawSummary));
  assert.equal(summary.schemaVersion, 1);
  assert.deepEqual(summary.compiler, compilerIdentity);
  assert.equal(summary.repository, manifest.repository);
  assert.equal(summary.revision, manifest.revision);
  assert.equal(summary.sourceRoot, fixture);
  assert.equal(summary.manifestSha256, sha256(read(root, "cases.json")));
  assert.deepEqual(JSON.parse(read(root, "cases.json")), manifest);
  const bundle = path.join(root, "compiler/compiler-sfc.esm-browser.mjs");
  assert.equal(sha256(fs.readFileSync(bundle)), compilerIdentity.sha256);
  const compiler = await import(pathToFileURL(bundle).href);
  assert.equal(compiler.version, compilerIdentity.version);
  assert.deepEqual(
    summary.licenses.map((item) => item.path),
    Object.keys(licensePins),
  );
  for (const item of summary.licenses) {
    const bytes = read(root, `licenses/${item.path}`);
    assert.equal(sha256(bytes), licensePins[item.path]);
    assert.equal(bytes.length, item.bytes);
    assert.deepEqual(bytes, read(fixture, item.path));
  }
  assert.equal(summary.records.length, 10);
  const files = [];
  const qualifications = [];
  function packet(directory, name) {
    const relative = `${directory}/${name}.json`;
    const bytes = read(root, relative);
    files.push({ path: relative, sha256: sha256(bytes) });
    return graph(JSON.parse(bytes));
  }
  function returned(directory, name, recipe, callback, extension, contentField) {
    const captured = packet(directory, name);
    assert.deepEqual(captured.recipe, plain(recipe), `official recipe: ${directory}/${name}`);
    assert.equal(captured.status, "returned", `official API refused: ${directory}/${name}`);
    const expected = attempt(callback);
    assert.deepEqual(captured.warnings, expected.warnings);
    const content = captured.result[contentField];
    assert.equal(content, expected.result[contentField], `complete official module: ${name}`);
    const bytes = read(root, `${directory}/${name}.${extension}`);
    assert.equal(bytes.toString("utf8"), content);
    files.push({ path: `${directory}/${name}.${extension}`, sha256: sha256(bytes) });
    return { captured, expected: expected.result };
  }
  for (const [index, item] of manifest.cases.entries()) {
    const record = summary.records[index];
    assert.equal(record.path, item.path);
    assert.equal(record.sha256, item.sha256);
    const directory = `cases/${String(index + 1).padStart(2, "0")}-${path.basename(item.path, ".vue")}`;
    assert.equal(record.directory, directory);
    const bytes = read(root, `${directory}/original.vue`);
    assert.equal(sha256(bytes), item.sha256);
    assert.deepEqual(bytes, read(fixture, item.path));
    assert.equal(record.bytes, bytes.length);
    const source = bytes.toString("utf8");
    const filename = path.join(fixture, item.path);
    const parsed = compiler.parse(source, { filename, sourceMap: true, ignoreEmpty: false });
    const storedParse = packet(directory, "parse");
    assert.deepEqual(storedParse, plain(parsed), `full original official parse: ${item.path}`);
    assert.deepEqual(record.parseErrors, []);
    assert.deepEqual(parsed.errors, []);
    const descriptor = parsed.descriptor;
    const template = read(root, `${directory}/template.txt`).toString("utf8");
    assert.equal(template, descriptor.template.content);
    const id = `data-v-${item.sha256.slice(0, 8)}`;
    const isTS = ["ts", "tsx"].includes(descriptor.scriptSetup?.lang ?? descriptor.script?.lang);
    const extension = isTS ? "ts" : "js";
    const observedNames = [];
    function templatePacket(sourceMap, mode, bindings) {
      const compilerOptions = bindings
        ? { ...options(mode), isTS, bindingMetadata: bindings }
        : options(mode);
      const recipe = {
        filename,
        id,
        sourceMap,
        scoped: mode.mode === "module" && descriptor.styles.some((style) => style.scoped),
        compilerOptions,
        ...(sourceMap ? { inMap: descriptor.template.map } : {}),
      };
      const name = `template-${bindings ? "real-bindings-" : ""}${mode.name}-map-${sourceMap}`;
      const { captured, expected } = returned(
        directory,
        name,
        recipe,
        () => compiler.compileTemplate({ source: template, ...recipe }),
        bindings ? extension : "js",
        "code",
      );
      for (const key of ["preamble", "source", "map", "tips"])
        assert.deepEqual(
          captured.result[key],
          plain(expected[key]),
          `complete official ${key}: ${name}`,
        );
      assert.ok(captured.result.ast && typeof captured.result.ast === "object");
      const errors = captured.result.errors.map(diagnostic);
      assert.deepEqual(errors, expected.errors.map(diagnostic));
      const declared = record.templates.find((entry) => entry.name === name);
      assert.equal(declared.status, "returned");
      assert.deepEqual(declared.warnings, captured.warnings);
      assert.deepEqual(declared.errors.map(diagnostic), errors);
      assert.deepEqual(
        errors,
        item.path.endsWith("/InstanceAiConfirmationPanel.vue") && !mode.prefixIdentifiers
          ? instanceErrors(sourceMap)
          : [],
        `official ordered mode diagnostics: ${name}`,
      );
      observedNames.push(name);
      qualifications.push({
        path: item.path,
        name,
        qualification: errors.length ? "diagnostic output" : "accepted template",
      });
    }
    for (const sourceMap of [false, true]) {
      for (const mode of modes) templatePacket(sourceMap, mode);
      const scriptOptions = { id, sourceMap, inlineTemplate: false, isProd: false };
      const script = returned(
        directory,
        `script-map-${sourceMap}`,
        scriptOptions,
        () => compiler.compileScript(descriptor, scriptOptions),
        extension,
        "content",
      );
      for (const key of ["type", "loc", "attrs", "lang", "setup", "bindings", "imports", "map"])
        assert.deepEqual(
          script.captured.result[key],
          plain(script.expected[key]),
          `official script ${key}`,
        );
      for (const mode of modes) templatePacket(sourceMap, mode, script.expected.bindings);
      const inlineOptions = {
        id,
        sourceMap,
        inlineTemplate: true,
        isProd: false,
        templateOptions: { sourceMap, compilerOptions: options(modes[1]) },
      };
      const inline = returned(
        directory,
        `component-inline-module-map-${sourceMap}`,
        inlineOptions,
        () => compiler.compileScript(descriptor, inlineOptions),
        extension,
        "content",
      );
      assert.equal(inline.captured.templateInlined, Boolean(descriptor.scriptSetup));
      for (const key of ["bindings", "map", "loc", "attrs", "imports"])
        assert.deepEqual(
          inline.captured.result[key],
          plain(inline.expected[key]),
          `official inline ${key}`,
        );
      for (const [styleIndex, style] of descriptor.styles.entries()) {
        const name = `style-${styleIndex}-map-${sourceMap}`;
        const captured = packet(directory, name);
        const styleRecipe = {
          filename,
          id,
          scoped: Boolean(style.scoped),
          modules: Boolean(style.module),
          preprocessLang: style.lang,
          inMap: sourceMap ? style.map : undefined,
          postcssOptions: { map: sourceMap ? { inline: false, annotation: false } : false },
        };
        assert.deepEqual(captured.recipe, plain(styleRecipe));
        if (style.lang && style.lang !== "css") {
          assert.equal(captured.status, "threw");
          assert.equal(
            captured.error.message,
            "[@vue/compiler-sfc] Style preprocessing in the browser build must provide the `preprocessCustomRequire` option to return the in-browser version of the preprocessor.",
          );
          qualifications.push({
            path: item.path,
            name,
            qualification: "preprocessor unavailable; no style acceptance credit",
          });
        } else {
          assert.equal(captured.status, "returned");
          assert.deepEqual(captured.result.errors, []);
          const expected = attempt(() =>
            compiler.compileStyle({ source: style.content, ...styleRecipe }),
          );
          assert.deepEqual(captured.warnings, expected.warnings);
          for (const key of ["code", "map", "errors", "dependencies", "modules"])
            assert.deepEqual(
              captured.result[key],
              plain(expected.result[key]),
              `official style ${key}`,
            );
        }
      }
    }
    assert.deepEqual(
      record.templates.map((entry) => entry.name),
      observedNames,
    );
    assert.ok(record.templates.every((entry) => entry.status === "returned"));
    assert.ok(record.scripts.every((entry) => entry.errors.length === 0));
    assert.ok(record.componentModules.every((entry) => entry.errors.length === 0));
    assert.equal(record.styles.length, descriptor.styles.length * 2);
    assert.deepEqual(
      record.scripts.map((entry) => [entry.name, entry.status]),
      [
        ["script-map-false", "returned"],
        ["script-map-true", "returned"],
      ],
    );
    assert.deepEqual(
      record.componentModules.map((entry) => [entry.name, entry.status]),
      [
        ["component-inline-module-map-false", "returned"],
        ["component-inline-module-map-true", "returned"],
      ],
    );
  }
  return {
    compiler: compilerIdentity,
    captureSha256: sha256(rawSummary),
    files,
    qualifications,
    rawMapDiagnosticAnomaly:
      "InstanceAi map=true shared error locations retain official lines 710/1246/1246; authored source locations remain 710/719/719.",
  };
}
