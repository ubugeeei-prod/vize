import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname } from "node:path";
import { join, relative, resolve } from "node:path";
import { parse } from "yaml";

const consumers =
  /release-input-scopes|release-public-acceptance\.test|release-retirement-transport-law/;

export function verifyWorkflowConsumers(document, file) {
  const inputs = ["workflow_dispatch", "workflow_call"].flatMap((kind) =>
    Object.values(document.on?.[kind]?.inputs ?? {}).map(({ default: value, options }) => ({
      value,
      options,
    })),
  );
  assert.doesNotMatch(JSON.stringify({ env: document.env, inputs }), consumers, file);
  for (const job of Object.values(document.jobs ?? {})) {
    const authority = {
      uses: job.uses,
      with: job.with,
      env: job.env,
      steps: job.steps?.map(({ run, uses, with: inputs, env }) => ({ run, uses, inputs, env })),
    };
    assert.doesNotMatch(JSON.stringify(authority), consumers, file);
  }
}

function physicalDirectory(directory, boundary) {
  for (let current = resolve(directory); ; current = dirname(current)) {
    assert.ok(lstatSync(current).isDirectory(), `physical consumer directory required: ${current}`);
    if (current === resolve(boundary)) return;
    assert.notEqual(
      current,
      dirname(current),
      "consumer directory must remain inside its boundary",
    );
  }
}

function verifyConsumerFile(file, boundary, reviewedFiles) {
  physicalDirectory(dirname(file), boundary);
  assert.ok(lstatSync(file).isFile(), `physical consumer source required: ${file}`);
  const source = readFileSync(file);
  if (consumers.test(source.toString("utf8")))
    assert.equal(
      createHash("sha256").update(source).digest("hex"),
      reviewedFiles[relative(boundary, file)],
      `reviewed consumer body required: ${file}`,
    );
}

export function verifyConsumerSources(directory, boundary = directory, reviewedFiles = {}) {
  let sources = 0;
  const visit = (directory) => {
    physicalDirectory(directory, boundary);
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (["node_modules", "target", "dist", ".artifacts", ".git"].includes(entry.name)) continue;
      const file = join(directory, entry.name);
      assert.ok(!entry.isSymbolicLink(), `linked consumer requires review: ${file}`);
      if (entry.isDirectory()) visit(file);
      else if (/\.(?:rs|[cm]?[jt]sx?|json|vue|sh|py|ya?ml|toml)$/.test(entry.name)) {
        assert.ok(entry.isFile(), `linked consumer requires review: ${file}`);
        verifyConsumerFile(file, boundary, reviewedFiles);
        sources++;
      }
    }
  };
  visit(directory);
  return sources;
}

export function verifyWorkflowSources(directory, boundary = directory) {
  physicalDirectory(directory, boundary);
  for (const file of readdirSync(directory)) {
    if (!/\.ya?ml$/.test(file)) continue;
    assert.ok(
      lstatSync(join(directory, file)).isFile(),
      `physical workflow source required: ${file}`,
    );
    verifyWorkflowConsumers(parse(readFileSync(join(directory, file), "utf8")), file);
  }
}

export function verifyNativeConsumerClosure(cwd, reviewedFiles) {
  let sources = 0;
  for (const directory of ["tools", ".github/actions", "crates", "davinci", "npm"])
    sources += verifyConsumerSources(join(cwd, directory), cwd, reviewedFiles);
  verifyConsumerFile(join(cwd, "package.json"), cwd, reviewedFiles);
  verifyWorkflowSources(join(cwd, ".github/workflows"), cwd);
  assert.ok(sources > 0, "actual consumer sources must be checked");
}

export function verifyConsumerRefusalControls(t, root, workflowPath, excluded, reviewedFiles) {
  const fixture = mkdtempSync(join(os.tmpdir(), "vize-native-consumers-"));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  const source = join(fixture, "native.mjs");
  writeFileSync(source, "export const native = true;\n");
  assert.equal(verifyConsumerSources(fixture), 1, "physical benign source is scanned");
  writeFileSync(source, `const source = ${JSON.stringify(excluded[0])};\n`);
  assert.throws(() => verifyConsumerSources(fixture));
  rmSync(source);
  const linked = join(fixture, "native-actions");
  symlinkSync(join(root, ".github/actions"), linked);
  assert.throws(() => verifyConsumerSources(fixture), /linked consumer requires review/);
  assert.throws(() => verifyConsumerSources(linked), /physical consumer directory required/);
  assert.throws(
    () => verifyConsumerSources(join(linked, "check-reference-path"), fixture),
    /physical consumer directory required/,
  );
  rmSync(linked);
  symlinkSync(join(root, workflowPath), join(fixture, "native.yml"));
  assert.throws(() => verifyWorkflowSources(fixture), /physical workflow source required/);
  rmSync(join(fixture, "native.yml"));
  copyFileSync(
    join(root, "tools/config/vite-plus/tooling-test-scopes.ts"),
    join(fixture, "unknown-metadata.ts"),
  );
  assert.throws(
    () => verifyConsumerSources(fixture, fixture, reviewedFiles),
    /reviewed consumer body/,
  );
}

export function prepareNativeConsumerFixture(directory, cwd, producer) {
  for (const folder of [
    ".github/actions",
    ".github/workflows",
    "crates",
    "davinci",
    "npm",
    dirname(producer),
  ])
    mkdirSync(join(directory, folder), { recursive: true });
  for (const file of ["package.json", producer])
    copyFileSync(join(cwd, file), join(directory, file));
}
