// Full source-built SFC CSS enters unchanged; Vue supplies independent CSS.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { CSSRule, Window } from "happy-dom";

const fixtureRoot = new URL(
  "../../_fixtures/differential/compiler/benchmark-mixed-style-scope/",
  import.meta.url,
);
const original = readFileSync(new URL("original.vue.txt", fixtureRoot));
const custody = JSON.parse(readFileSync(new URL("source-custody.json", fixtureRoot)));
const manifest = JSON.parse(
  readFileSync(new URL("../benchmark-mixed-style-scope.manifest.json", fixtureRoot)),
);
const pinned = manifest.cases[0];
for (const artifact of [
  ...pinned.inputs.files.map((input) => ({
    ...input,
    path: pinned.inputs.root + "/" + input.path,
  })),
  pinned.reference,
]) {
  const bytes = readFileSync(new URL("../" + artifact.path, fixtureRoot));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), artifact.sha256, artifact.path);
}
assert.equal(custody.sourceCommit, "5489aee433cd1054b9d72973457498544da7c467");
assert.equal(
  custody.sourceSha256,
  "78c44890f7d99f3716c9291712f962bf5fcf4b09efbe2f020a92b3cd6778d35b",
);
assert.equal(createHash("sha256").update(original).digest("hex"), custody.sourceSha256);
assert.equal(original.length, custody.sourceBytes);
const require = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vue = require("vue");
const { parse, compileStyle } = require("vue/compiler-sfc");
assert.match(vue.version, /^3\.5\./, "use the repository's pinned official stable Vue oracle");
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const observations = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(observations.length, 24, "four full inputs × three backends × two aggregate flags");
const identities = new Set();
const expectedIdentities = new Set(
  ["original", "reordered", "ordinary-only", "scoped-only"].flatMap((name) =>
    ["dom", "ssr", "vapor"].flatMap((backend) =>
      [false, true].map((aggregate) => name + "/" + backend + "/" + aggregate),
    ),
  ),
);

function selectors(rules) {
  return Array.from(rules, (rule) => {
    if (rule.type === CSSRule.MEDIA_RULE) {
      assert.equal(typeof rule.conditionText, "string", "the authored media constraint must parse");
      const condition = rule.conditionText
        .replace(/min-width\s*:\s*1px/g, "width >= 1px")
        .replace(/\s/g, "");
      return [condition, selectors(rule.cssRules)];
    }
    assert.equal(
      rule.type,
      CSSRule.STYLE_RULE,
      "the original plant contains only style/media rules",
    );
    assert.equal(rule.cssRules.length, 0, "the original style rules contain no nested rules");
    return rule.selectorText;
  });
}

function observe(css) {
  const window = new Window();
  const { document } = window;
  const style = document.createElement("style");
  style.textContent = css;
  document.head.append(style);
  document.body.innerHTML =
    '<div class="cascade tail" data-v-abc12345></div><div class="cascade tail"></div><div class="cascade" data-v-abc12345></div><div class="tail"></div>';
  assert.ok(style.sheet, "actual generated stylesheet must parse");
  const result = {
    selectors: selectors(style.sheet.cssRules),
    elements: Array.from(document.body.children, (element) => {
      const computed = window.getComputedStyle(element);
      return ["color", "border-top-width", "border-top-style", "border-top-color", "--gap"].map(
        (property) => [property, computed.getPropertyValue(property)],
      );
    }),
  };
  void window.happyDOM.abort();
  return result;
}

for (const observation of observations) {
  const identity = observation.name + "/" + observation.backend + "/" + observation.aggregateScoped;
  assert.ok(expectedIdentities.has(identity), identity);
  assert.ok(!identities.has(identity), identity);
  identities.add(identity);
  if (observation.name === "original") assert.equal(observation.source, original.toString("utf8"));
  const parsed = parse(observation.source);
  assert.deepEqual(parsed.errors, [], identity);
  const references = parsed.descriptor.styles.map((block) => {
    const result = compileStyle({
      source: block.content,
      id: "data-v-abc12345",
      scoped: block.scoped,
    });
    assert.deepEqual(result.errors, [], identity);
    return result.code;
  });
  const reference = references.join("\n");
  const actual = observe(observation.css);
  assert.deepEqual(
    actual,
    observe(reference),
    identity + ": complete rule order and CSS DOM scope",
  );
  const flattened = JSON.stringify(actual.selectors);
  if (parsed.descriptor.styles.some((block) => !block.scoped)) {
    assert.ok(flattened.includes('".tail"'), identity + ": ordinary block stays global");
    assert.ok(
      !flattened.includes(".tail[data-v-"),
      identity + ": no scope leaks into ordinary block",
    );
  }
  console.log(
    JSON.stringify({
      identity,
      vue: vue.version,
      css: observation.css,
      reference,
      observation: actual,
    }),
  );
}
assert.equal(identities.size, 24);
assert.deepEqual(identities, expectedIdentities);
