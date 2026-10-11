import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { prepareSeparateLintInstallGraph } from "../_helpers/lint-fixture-install-graph.ts";
import {
  assertPinnedFixtureFiles,
  readPinnedFixtureSource,
  sha256,
} from "../_helpers/pinned-fixture-source.ts";

function withGraph(run: (source: string, install: string, closure: string, pin: string) => void) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-graph-"));
  const source = path.join(temporary, "source");
  const install = path.join(temporary, "install");
  const closure = path.join(temporary, "closure");
  try {
    for (const dir of [source, install, closure]) fs.mkdirSync(dir);
    const pkg =
      '{"packageManager":"pnpm@11.20.0","dependencies":{"@lunariajs/core":"https://example.com/missing.tgz"}}\n';
    const lock = "original unavailable preview integrity\n";
    for (const dir of [source, install]) {
      fs.writeFileSync(path.join(dir, "Original.vue"), "<template><div /></template>\n");
      fs.writeFileSync(path.join(dir, "package.json"), pkg);
      fs.writeFileSync(path.join(dir, "pnpm-lock.yaml"), lock);
    }
    execFileSync("git", ["init", "-q"], { cwd: source });
    execFileSync("git", ["add", "."], { cwd: source });
    execFileSync(
      "git",
      ["-c", "user.name=Vize", "-c", "user.email=vize@example.com", "commit", "-qm", "pin"],
      { cwd: source },
    );
    const pin = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: source,
      encoding: "utf8",
    }).trim();
    const effective = pkg.replace('"https://example.com/missing.tgz"', '"0.2.0"');
    fs.writeFileSync(path.join(closure, "package.json"), effective);
    fs.writeFileSync(
      path.join(closure, "pnpm-lock.yaml"),
      "reviewed stable dependency integrity\n",
    );
    fs.writeFileSync(
      path.join(closure, "source.json"),
      JSON.stringify({
        revision: pin,
        originalFrozenClosure: false,
        inputs: { "package.json": sha256(pkg), "pnpm-lock.yaml": sha256(lock) },
        effectiveInputs: {
          "package.json": sha256(effective),
          "pnpm-lock.yaml": sha256("reviewed stable dependency integrity\n"),
        },
        originalDependency: {
          name: "@lunariajs/core",
          specifier: "https://example.com/missing.tgz",
        },
        replacement: { name: "@lunariajs/core", specifier: "0.2.0" },
      }),
    );
    run(source, install, closure, pin);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}

test("a separate installer changes two explicit inputs while lint retains every original blob", () => {
  withGraph((root, install, closure, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    const graph = prepareSeparateLintInstallGraph(root, install, source, closure);
    assert.equal(graph.receipt.originalFrozenClosure, false);
    assertPinnedFixtureFiles(root, source);
    assert.throws(() => graph.attach(), /ENOENT/);
    fs.mkdirSync(path.join(install, "node_modules"));
    fs.writeFileSync(path.join(install, "node_modules", "dependency.txt"), "installed");
    graph.attach();
    assert.equal(
      fs.readFileSync(path.join(root, "node_modules", "dependency.txt"), "utf8"),
      "installed",
    );
    graph.verify();
    fs.appendFileSync(path.join(install, "Original.vue"), "changed");
    assert.throws(() => graph.verify(), /original fixture bytes changed/);
  });
});

test("a sidecar cannot admit a second manifest change by updating its checksum", () => {
  withGraph((root, install, closure, pin) => {
    const filename = path.join(closure, "package.json");
    fs.appendFileSync(filename, " ");
    const metadataPath = path.join(closure, "source.json");
    const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf8"));
    metadata.effectiveInputs["package.json"] = sha256(fs.readFileSync(filename));
    fs.writeFileSync(metadataPath, JSON.stringify(metadata));
    assert.throws(
      () =>
        prepareSeparateLintInstallGraph(root, install, readPinnedFixtureSource(root, pin), closure),
      /only the explicitly reviewed/,
    );
  });
});

test("migration rejects a different source pin and source/installer alias", () => {
  withGraph((root, install, closure, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    assert.throws(() => prepareSeparateLintInstallGraph(root, root, source, closure));
    const filename = path.join(closure, "source.json");
    const metadata = JSON.parse(fs.readFileSync(filename, "utf8"));
    metadata.revision = "0".repeat(40);
    fs.writeFileSync(filename, JSON.stringify(metadata));
    assert.throws(
      () => prepareSeparateLintInstallGraph(root, install, source, closure),
      /authored source pin/,
    );
  });
});
