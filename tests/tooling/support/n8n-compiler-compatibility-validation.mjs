// Qualify complete historical parser notice vectors without suppressing any output.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { read, sha256 } from "./n8n-compiler-custody-receipt.mjs";

const message =
  "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.";
const compilerSha256 = "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec";
const instance =
  "packages/frontend/editor-ui/src/features/ai/instanceAi/components/InstanceAiConfirmationPanel.vue";
const families = new Map([
  [
    "packages/frontend/@n8n/design-system/src/components/N8nFormInput/FormInput.vue",
    [
      ["span", 2127, 2169],
      ["span", 2430, 2469],
    ],
  ],
  [
    "packages/frontend/editor-ui/src/app/dev/dev-panel/DevPanel.vue",
    [
      ["div", 78, 174],
      ["div", 178, 383],
      ["div", 387, 652],
      ["span", 5857, 5918],
    ],
  ],
  [
    "packages/frontend/editor-ui/src/features/workflows/workflowHistory/components/WorkflowHistoryVersionSelect.vue",
    [["div", 36, 65]],
  ],
]);
const voidTags = new Set(
  "area,base,br,col,embed,hr,img,input,link,meta,param,source,track,wbr".split(","),
);

function originalSpans(compiler, originalSource) {
  const parsed = compiler.parse(originalSource, { sourceMap: true, ignoreEmpty: false });
  assert.deepEqual(parsed.errors, []);
  const template = parsed.descriptor.template;
  assert.ok(template?.ast);
  const spans = [];
  function visit(node) {
    if (
      node.type === 1 &&
      node.ns === 0 &&
      node.tagType === 0 &&
      node.isSelfClosing &&
      !voidTags.has(node.tag)
    ) {
      assert.equal(
        originalSource.slice(node.loc.start.offset, node.loc.end.offset),
        node.loc.source,
      );
      assert.ok(node.loc.source.startsWith(`<${node.tag}`) && node.loc.source.endsWith("/>"));
      spans.push([
        node.tag,
        Buffer.byteLength(originalSource.slice(template.loc.start.offset, node.loc.start.offset)),
        Buffer.byteLength(originalSource.slice(template.loc.start.offset, node.loc.end.offset)),
      ]);
    }
    for (const child of node.children ?? []) visit(child);
  }
  visit(template.ast);
  return spans;
}

export async function validateCompatibilityNotices(beforeRoot, afterRoot, officialRoot, manifest) {
  const bundle = path.join(officialRoot, "compiler/compiler-sfc.esm-browser.mjs");
  assert.equal(sha256(fs.readFileSync(bundle)), compilerSha256);
  const compiler = await import(pathToFileURL(bundle).href);
  assert.equal(compiler.version, "3.5.26");
  const official = JSON.parse(read(officialRoot, "capture.json"));
  const comparisons = [];
  for (const item of manifest.cases) {
    if (item.path === instance) continue;
    const filename = `${path.basename(item.path)}.json`;
    const before = JSON.parse(read(beforeRoot, filename));
    const after = JSON.parse(read(afterRoot, filename));
    const record = official.records.find((entry) => entry.path === item.path);
    assert.ok(record);
    const original = read(officialRoot, `${record.directory}/original.vue`);
    assert.equal(sha256(original), item.sha256);
    assert.equal(before.originalSource, original.toString("utf8"));
    assert.equal(after.originalSource, before.originalSource);
    const spans = originalSpans(compiler, before.originalSource);
    const expectedSpans = families.get(item.path) ?? [];
    assert.deepEqual(spans, expectedSpans, `independent original self-closing spans: ${item.path}`);
    const errors = expectedSpans.map(([, start, end]) => ({
      code: "ExtendPoint",
      location: { span: { end, start } },
      message,
    }));
    for (const prefixIdentifiers of [false, true]) {
      const row = (packet) =>
        packet.rows.find(
          (entry) => entry.kind === "template" && entry.prefixIdentifiers === prefixIdentifiers,
        );
      assert.deepEqual(
        row(before)?.legacy.errors,
        errors,
        `complete historical notices: ${item.path}`,
      );
      assert.deepEqual(row(after)?.legacy.errors, errors, `complete current notices: ${item.path}`);
      comparisons.push({
        path: item.path,
        prefixIdentifiers,
        independentOriginalSpans: spans,
        beforeErrors: row(before).legacy.errors,
        afterErrors: row(after).legacy.errors,
        beforePacketSha256: sha256(read(beforeRoot, filename)),
        afterPacketSha256: sha256(read(afterRoot, filename)),
      });
    }
  }
  assert.equal(comparisons.length, 18);
  assert.equal(
    comparisons.reduce((count, row) => count + row.beforeErrors.length, 0),
    14,
  );
  return comparisons;
}

export function validateTemplateNotices(row, originalPath, comparisons) {
  const qualified = comparisons.find(
    (item) => item.path === originalPath && item.prefixIdentifiers === row.prefixIdentifiers,
  );
  assert.ok(qualified, `original parser notice contract is not qualified: ${originalPath}`);
  assert.deepEqual(row.legacy.errors, qualified.afterErrors);
  assert.deepEqual(row.legacy.errors, qualified.beforeErrors);
}
