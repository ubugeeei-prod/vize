import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Reconstruct only the prior generated initializer and its generated offsets.
// Actual current text/maps remain raw and frozen by the complete TS-40 snapshots.
const root = process.env.VIZE_TEMPLATE_EMIT_TS40_CAPTURE;
const baseline = JSON.parse(
  readFileSync(
    "tests/_fixtures/differential/typechecker/template-dollar-emit/projection-baseline.json",
    "utf8",
  ),
);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

const navigationDeclaration =
  "  type __VizeEmptyVueNavigationSlots<C> = ('__isFragment' | '__isTeleport' | '__isSuspense' | 'slots') extends keyof C ? __VizeIsAny<C[Extract<'__isFragment' | '__isTeleport' | '__isSuspense', keyof C>]> extends true ? any : [NonNullable<C[Extract<'__isFragment' | '__isTeleport' | '__isSuspense', keyof C>]>] extends [never] ? C extends { slots?: infer __S } ? __VizeIsAny<__S> extends true ? any : unknown extends __S ? any : [keyof NonNullable<__S>] extends [never] ? C extends { setup?: (props: infer __P, ...args: any[]) => any } ? string extends keyof __P ? any : {} : {} : any : any : any : any;\n  type __VizeNativeNavigationSlots<C> = C extends new (...args: any[]) => { $slots: any } ? __VizeIsAny<InstanceType<C>> extends true ? __VizeEmptyVueNavigationSlots<C> : __VizeIsAny<InstanceType<C>['$slots']> extends true ? any : unknown extends InstanceType<C>['$slots'] ? any : InstanceType<C>['$slots'] : any;\n  type __VizeNavigationSlots<C> = __VizeIsAny<C> extends true ? any : C extends { readonly __vizeSlots?: infer __S } ? __VizeIsAny<__S> extends true ? __S : unknown extends __S ? [keyof NonNullable<__S>] extends [never] ? __VizeNativeNavigationSlots<C> : NonNullable<__S> : NonNullable<__S> : __VizeNativeNavigationSlots<C>;\n";
const navigationNames = [
  "__VizeEmptyVueNavigationSlots",
  "__VizeNativeNavigationSlots",
  "__VizeNavigationSlots",
];
const baselineBytes = readFileSync(
  "tests/_fixtures/differential/typechecker/template-dollar-emit/projection-baseline.json",
);
assert.equal(
  hash(baselineBytes),
  "e7fcc4e2239ce80fb0081b8fd075aa7bcf52339de5787e439b821dcdbb61e1b9",
);
assert.deepEqual(baseline, JSON.parse(baselineBytes));
const read = (...parts) => readFileSync(join(root, ...parts), "utf8");
const byteLength = (text) => Buffer.byteLength(text);
const oldDeclaration = "const $emit = __ctx.$emit;";
const declarations = {
  "dual-scripts-emits":
    "const $emit = undefined as unknown as __EmitFn<{ submit: [value: string] }>;",
  "define-model":
    'const $emit = undefined as unknown as __EmitFn<{\n"update:title": [value: string];\n"update:modelValue": [value: number];\n}>;',
};
const digest = (text, bytes = false) =>
  `${bytes ? byteLength(text) : text === "" ? 0 : text.split("\n").length}:${hash(text)}`;

function spanDigest(text) {
  let depth = 0;
  let count = text === "" ? 0 : 1;
  for (const character of text) {
    if (character === "[") depth += 1;
    if (character === "]") depth -= 1;
    assert(depth >= 0);
    if (character === "\n" && depth === 0) count += 1;
  }
  assert.equal(depth, 0);
  return `${count}:${hash(text)}`;
}

function rewindInitializer(text, declaration, allowAbsent = false) {
  const index = text.indexOf(declaration);
  if (allowAbsent && index === -1) {
    assert(!text.includes("const $emit = "));
    return { text, cut: Infinity, end: Infinity, delta: 0 };
  }
  assert(index >= 0, "the independently authored typed initializer is absent");
  assert.equal(text.indexOf(declaration, index + declaration.length), -1);
  const cut = byteLength(text.slice(0, index));
  return {
    text: text.slice(0, index) + oldDeclaration + text.slice(index + declaration.length),
    cut,
    end: cut + byteLength(declaration),
    delta: byteLength(declaration) - byteLength(oldDeclaration),
  };
}

function initializerOffset(value, edit) {
  const number = Number(value);
  if (number <= edit.cut) return number;
  assert(number >= edit.end, "an authored mapping endpoint entered the synthetic initializer");
  return number - edit.delta;
}

function generatedPairs(text, edit) {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/g,
    (_match, start, end, source, sourceEnd) =>
      `${offset(start, edit)}:${offset(end, edit)}>${source}:${sourceEnd}`,
  );
}

function targetPairs(text, edit) {
  return text.replace(
    /(\d+):(\d+)>(\d+):(\d+)/g,
    (_match, source, sourceEnd, start, end) =>
      `${source}:${sourceEnd}>${offset(start, edit)}:${offset(end, edit)}`,
  );
}

function intervals(text, edit) {
  return text.replace(/(\d+):(\d+)>(\d+):(\d+)/g, (_match, start, length, source, sourceLength) => {
    const previous = offset(start, edit);
    return `${previous}:${offset(Number(start) + Number(length), edit) - previous}>${source}:${sourceLength}`;
  });
}

function rewind(text, declaration, allowAbsent = false) {
  assert(
    declaration === declarations["define-model"] ||
      declaration === declarations["dual-scripts-emits"],
    "unknown initializer family",
  );
  let navigation = { text, cut: Infinity, end: Infinity, delta: 0 };
  const isModel = declaration === declarations["define-model"];
  if (isModel && !(allowAbsent && !text.includes(declaration))) {
    const index = text.indexOf(navigationDeclaration);
    assert(index >= 0, "the complete current navigation declaration is absent");
    assert.equal(text.indexOf(navigationDeclaration, index + navigationDeclaration.length), -1);
    const restored = text.slice(0, index) + text.slice(index + navigationDeclaration.length);
    assert(navigationNames.every((name) => !restored.includes(name)));
    const cut = byteLength(text.slice(0, index));
    const delta = Buffer.byteLength(navigationDeclaration);
    navigation = { text: restored, cut, end: cut + delta, delta };
  } else {
    assert(navigationNames.every((name) => !text.includes(name)));
  }
  return { ...rewindInitializer(navigation.text, declaration, allowAbsent), navigation };
}

function offset(value, edit) {
  const number = Number(value);
  assert(Number.isSafeInteger(number) && number >= 0);
  assert(
    number < edit.navigation.cut || number >= edit.navigation.end,
    "an authored endpoint entered the current navigation declaration",
  );
  return initializerOffset(initializerOffset(value, edit.navigation), edit);
}

const proofs = [];
for (const [name, expected] of Object.entries(baseline)) {
  assert.equal(hash(read(name, "original.vue")), expected["source-sha256"]);
  const declaration = declarations[name];
  assert(declaration);
  const canon = rewind(read(name, "canon", "text.ts"), declaration);
  const preRewrite = rewind(read(name, "canon", "pre-rewrite-text.ts"), declaration);
  assert.equal(canon.delta, name === "dual-scripts-emits" ? 50 : 97);
  assert.equal(digest(canon.text, true), expected.canon.text);
  assert.equal(digest(preRewrite.text, true), expected.canon["pre-rewrite-text"]);
  assert.equal(
    spanDigest(generatedPairs(read(name, "canon", "mappings.txt"), canon)),
    expected.canon.mappings,
  );
  assert.equal(
    digest(
      read(name, "canon", "semantic-links.txt").replace(
        /(\d+):(\d+)>(\d+):(\d+)/g,
        (_match, start, end, target, targetEnd) =>
          `${offset(start, canon)}:${offset(end, canon)}>${offset(target, canon)}:${offset(targetEnd, canon)}`,
      ),
    ),
    expected.canon["semantic-links"],
  );
  assert.equal(
    `0:${hash(read(name, "canon", "import-map.txt"))}`,
    expected.canon["import-source-map"],
  );
  const probes = (original, virtual) =>
    Array.from({ length: original + 1 }, (_value, index) => `o${index}>${index};`).join("") +
    Array.from({ length: virtual + 1 }, (_value, index) => `v${index}>${index};`).join("");
  const current = read(name, "canon", "text.ts");
  const currentPreRewrite = read(name, "canon", "pre-rewrite-text.ts");
  assert.equal(
    read(name, "canon", "import-probes.txt"),
    probes(byteLength(currentPreRewrite), byteLength(current)),
  );
  const previousProbes = probes(byteLength(preRewrite.text), byteLength(canon.text));
  assert.equal(
    `${byteLength(preRewrite.text) + byteLength(canon.text) + 2}:${hash(previousProbes)}`,
    expected.canon["import-source-map-probes"],
  );

  const mapper = rewind(read(name, "content-mapper", "text.ts"), declaration);
  assert.equal(digest(mapper.text, true), expected["content-mapper"].text);
  const mapperMappings = read(name, "content-mapper", "mappings.txt")
    .split("\n")
    .map((line) => {
      const row = JSON.parse(line);
      const [start, length, ...unchanged] = row;
      const previous = offset(start, mapper);
      return `[${[previous, offset(start + length, mapper) - previous, ...unchanged].join(", ")}]`;
    })
    .join("\n");
  assert.equal(digest(mapperMappings), expected["content-mapper"].mappings);
  const mapperLinks = read(name, "content-mapper", "semantic-links.txt").replace(
    /(\d+):(\d+)>(\d+):(\d+):(\d+)/g,
    (_match, source, sourceLength, target, length, kind) => {
      const previous = offset(target, mapper);
      return `${source}:${sourceLength}>${previous}:${offset(Number(target) + Number(length), mapper) - previous}:${kind}`;
    },
  );
  assert.equal(digest(mapperLinks), expected["content-mapper"]["semantic-links"]);
  assert.equal(
    digest(read(name, "content-mapper", "diagnostics.txt")),
    expected["content-mapper"].diagnostics,
  );
  assert.equal(
    digest(intervals(read(name, "content-mapper", "authored-hits.txt"), mapper)),
    expected["content-mapper"]["authored-hits"],
  );

  const documents = read(name, "maestro", "text.txt").split("\n");
  const edits = new Map();
  let previousBytes = 0;
  const previousDocuments = documents.map((line, index) => {
    const [uri, language, size, sha] = line.split("|");
    const text = read(name, `maestro-document-${index}`, "text.ts");
    assert.equal(`${byteLength(text)}:${hash(text)}`, `${size}:${sha}`);
    const edit = rewind(text, declaration, true);
    edits.set(uri, edit);
    previousBytes += byteLength(edit.text);
    return `${uri}|${language}|${byteLength(edit.text)}|${hash(edit.text)}`;
  });
  assert.equal([...edits.values()].filter((edit) => edit.delta > 0).length, 1);
  assert.equal(`${previousBytes}:${hash(previousDocuments.join("\n"))}`, expected.maestro.text);
  function maestroRows(file, authored) {
    return read(name, "maestro", file)
      .split("\n")
      .map((line) => {
        const parts = line.split("|");
        const uri = authored ? parts[1] : parts[0];
        const edit = edits.get(uri);
        assert(edit);
        parts[2] = targetPairs(parts[2], edit);
        return parts.join("|");
      })
      .join("\n");
  }
  assert.equal(digest(maestroRows("mappings.txt", false)), expected.maestro.mappings);
  assert.equal(digest(maestroRows("authored-hits.txt", true)), expected.maestro["authored-hits"]);
  proofs.push({
    name,
    sourceSha256: expected["source-sha256"],
    initializerBytesAdded: canon.delta,
    restoredPriorTextAndEveryMapping: true,
  });
}
const files = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? files(join(dir, entry.name)) : [join(dir, entry.name)],
  );
writeFileSync(
  join(root, "receipts.json"),
  JSON.stringify(
    {
      sourceSha: process.env.SOURCE_SHA,
      verifierSha256: hash(readFileSync(new URL(import.meta.url))),
      proofs,
      completeRawInputsAndOutputs: files(root).map((path) => ({
        path: path.slice(root.length + 1),
        sha256: hash(readFileSync(path)),
      })),
    },
    null,
    2,
  ) + "\n",
);
