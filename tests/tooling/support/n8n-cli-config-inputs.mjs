import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const projection = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
);

export function validateProjection(value = projection) {
  assert.equal(value.schema, "vize.n8n.cli-config-acceptance");
  assert.equal(value.version, 1);
  assert.equal(value.requirements.revision, "5c2a2cf3d837b6538a3330a4d7cc817e4b6f04e2");
  assert.equal(value.fixtureRevision, "e882e8a483f433facb47bab9b407d0ec00a81172");
  assert.equal(value.linter.enabled, true);
  assert.equal(value.linter.preset, "incremental");
  const selected = Object.keys(value.linter.rules).sort();
  assert.equal(selected.length, 51);
  assert.ok(Object.values(value.linter.rules).every((severity) => severity === "error"));
  assert.deepEqual(Object.keys(value.oracleRules).sort(), selected);
  const aliases = {
    "vue/require-component-registration": "vue/no-undef-components",
    "vue/sfc-element-order": "vue/block-order",
  };
  for (const name of selected) {
    const expected =
      aliases[name] ?? (name.startsWith("script/") ? name.replace("script/", "vue/") : name);
    assert.equal(value.oracleRules[name], expected, name);
  }
  assert.equal(new Set(Object.values(value.oracleRules)).size, 51);
  assert.deepEqual(value.linter.ruleOptions, {
    "vue/attribute-hyphenation": "always",
    "vue/component-name-in-template-casing": { casing: "PascalCase" },
    "vue/sfc-element-order": { order: ["script", "template", "style"] },
  });
  assert.equal(Object.keys(value.packageVueFileCounts).length, 9);
  assert.equal(
    Object.values(value.packageVueFileCounts).reduce((sum, count) => sum + count, 0),
    1369,
  );
  assert.equal(value.files, 1369);
  assert.equal(value.scriptlessFiles, 19);
  assert.equal(
    Object.values(value.scopes).flatMap(({ entries }) => entries.flatMap(({ files }) => files))
      .length,
    6,
  );
  return value;
}

export function scopedConfig(packageRoot, scoped) {
  const linter = structuredClone(projection.linter);
  const scope = scoped ? projection.scopes[packageRoot] : undefined;
  if (scope?.rules) Object.assign(linter.rules, scope.rules);
  return { linter, ...(scope?.entries ? { entries: structuredClone(scope.entries) } : {}) };
}

export function expectedScopedPacket(packet, packageRoot) {
  const scope = projection.scopes[packageRoot];
  const off = new Set(
    (scope?.entries ?? [])
      .filter(({ files }) => files.includes(packet.file))
      .flatMap(({ linter }) => Object.entries(linter.rules))
      .filter(([, severity]) => severity === "off")
      .map(([name]) => name),
  );
  const messages = packet.messages
    .filter(({ ruleId }) => !off.has(ruleId))
    .map((message) => {
      const severity = scope?.rules?.[message.ruleId];
      return severity === "warn" ? { ...message, severity: 1 } : message;
    });
  return {
    ...packet,
    messages,
    errorCount: messages.filter(({ severity }) => severity === 2).length,
    warningCount: messages.filter(({ severity }) => severity === 1).length,
  };
}
