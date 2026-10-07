import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, globSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = fileURLToPath(new URL("../../", import.meta.url));
const readJson = (file) => JSON.parse(readFileSync(join(root, file), "utf8"));
const manifest = readJson("tests/_fixtures/n8n-adoption.json");
const registry = readJson("tests/_fixtures/vue-ecosystem-fixtures.json");
const project = registry.projects.find(({ id }) => id === "n8n");
const fixture = join(root, project.fixturePath);

void test("n8n fixture pins licensed master separately from adoption requirements", () => {
  assert.equal(project.revision, "e882e8a483f433facb47bab9b407d0ec00a81172");
  assert.equal(manifest.fixtureRevision, project.revision);
  assert.equal(manifest.adoption.revision, "aa173be0c65c0646a7fcec32d2c18e1eaacbc8ff");
  assert.notEqual(manifest.adoption.revision, manifest.fixtureRevision);
  assert.deepEqual(project.vueGlobs, ["packages/**/*.vue"]);
  assert.deepEqual(project.coverage, registry.requiredToolCoverage);
  assert.deepEqual(project.license.files, ["LICENSE.md", "LICENSE_EE.md"]);
  const gitlink = execFileSync("git", ["ls-files", "--stage", project.fixturePath], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(gitlink, `160000 ${project.revision} 0\t${project.fixturePath}\n`);
  const shallow = execFileSync(
    "git",
    ["config", "-f", ".gitmodules", "--get", `submodule.${project.fixturePath}.shallow`],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(shallow, "true\n");
});

void test("n8n acceptance retains its complete custom rules and scoped overrides", () => {
  const { rules, packageOverrides, settings } = manifest.adoption;
  assert.equal(Object.keys(rules).length, 51);
  assert.deepEqual(settings, { vize: { preset: "incremental", helpLevel: "none" } });
  assert.deepEqual(rules["vize/vue/attribute-hyphenation"], ["error", "always"]);
  assert.deepEqual(rules["vize/vue/component-name-in-template-casing"], ["error", "PascalCase"]);
  assert.deepEqual(rules["vize/vue/sfc-element-order"], [
    "error",
    { order: ["script", "template", "style"] },
  ]);
  assert.equal(packageOverrides["editor-ui"]["vize/vue/attribute-hyphenation"], "warn");
  assert.equal(packageOverrides["editor-ui"]["vize/vue/no-multiple-template-root"].files.length, 5);
  assert.deepEqual(packageOverrides["design-system"]["vize/vue/require-v-for-key"], {
    severity: "off",
    files: ["src/components/N8nDatatable/Datatable.vue"],
  });
  assert.equal(manifest.adoption.retiredRules.length, 8);
  assert.ok(manifest.adoption.retiredRules.includes("no-deprecated-filter"));
});

void test("hydrated n8n corpus includes every Vue package and scriptless SFC", (t) => {
  if (!existsSync(join(fixture, "package.json"))) {
    assert.notEqual(process.env.VIZE_N8N_FIXTURE_REQUIRED, "1", "n8n fixture must be hydrated");
    t.skip("n8n fixture is not hydrated outside its Actions lane");
    return;
  }
  const head = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: fixture,
    encoding: "utf8",
  }).trim();
  assert.equal(head, manifest.fixtureRevision);
  const files = globSync(project.vueGlobs, { cwd: fixture }).sort();
  assert.equal(files.length, manifest.corpus.vueFileCount);
  const scriptless = files.filter(
    (file) => !/^\s*<script\b/m.test(readFileSync(join(fixture, file), "utf8")),
  );
  assert.deepEqual(scriptless, manifest.corpus.scriptlessFiles);
  assert.equal(scriptless.length, 19);
  const packageCounts = Object.fromEntries(
    Object.keys(manifest.corpus.packageVueFileCounts).map((prefix) => [
      prefix,
      files.filter((file) => file.startsWith(`${prefix}/`)).length,
    ]),
  );
  assert.deepEqual(packageCounts, manifest.corpus.packageVueFileCounts);
  assert.equal(
    Object.values(packageCounts).reduce((sum, count) => sum + count, 0),
    files.length,
  );
});
