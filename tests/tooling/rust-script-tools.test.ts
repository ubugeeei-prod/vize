import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("tool command surface is Rust Script first", () => {
  const source = readRepoFile("tools/commands/ci/verify-tool-layout.rs");
  assert.match(source, /rust-script tools: verified/);
  assert.match(source, /legacy command runner source must not come back/);
  assert.match(source, /tool host runner source must not come back/);
  assert.match(source, /collect_javascript_tools/);
  assert.match(source, /must be ported to Rust Script/);

  const commands = collectFiles(path.join(root, "tools", "commands")).filter(
    (file) =>
      file.endsWith(".rs") &&
      fs.readFileSync(file, "utf8").startsWith("#!/usr/bin/env rust-script\n"),
  );
  assert.ok(commands.length >= 40, "expected Rust Script command surface");

  for (const command of commands) {
    const relative = normalize(path.relative(root, command));
    const text = fs.readFileSync(command, "utf8");
    assert.match(text, /^#!\/usr\/bin\/env rust-script\n/, relative);
    assert.doesNotMatch(text, /legacy_command/, relative);
    assert.doesNotMatch(text, /tool_host::run\(/, relative);
    assert.doesNotMatch(text, /tool_host::Runtime::Node/, relative);
    assert.doesNotMatch(relative, /-vize\//, "editor command buckets use neutral names");
  }
  const result = verify(root);
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  assert.equal(
    result.stdout.trim(),
    `rust-script tools: verified ${commands.length} Rust Script commands`,
  );
});

test("ordinary tool modules must be reachable and resolve without ambiguity", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tool-modules-"));
  const commands = path.join(fixture, "tools/commands");
  try {
    fs.mkdirSync(commands, { recursive: true });
    fs.writeFileSync(path.join(fixture, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(fixture, "pnpm-workspace.yaml"), "packages: []\n");
    fs.writeFileSync(
      path.join(commands, "main.rs"),
      '#!/usr/bin/env rust-script\nmod support;\nmod nested { mod r#type; }\nconst EXAMPLE: &str = "mod absent;";\nfn main() {}\n',
    );
    fs.writeFileSync(path.join(commands, "support.rs"), "mod child;\n");
    fs.mkdirSync(path.join(commands, "support"));
    fs.writeFileSync(path.join(commands, "support/child.rs"), "");
    fs.mkdirSync(path.join(commands, "nested"));
    fs.writeFileSync(path.join(commands, "nested/type.rs"), "");
    assert.equal(verify(fixture).status, 0);
    const closure = spawnSync(
      "rust-script",
      [
        path.join(root, "tools/commands/ci/verify-tool-layout.rs"),
        "--module-closure",
        "tools/commands/main.rs",
      ],
      { cwd: fixture, encoding: "utf8" },
    );
    assert.equal(closure.status, 0, closure.stderr);
    assert.deepEqual(JSON.parse(closure.stdout), [
      "tools/commands/main.rs",
      "tools/commands/nested/type.rs",
      "tools/commands/support.rs",
      "tools/commands/support/child.rs",
    ]);
    fs.writeFileSync(path.join(commands, "orphan.rs"), "fn main() {}\n");
    const orphan = verify(fixture);
    assert.equal(orphan.status, 1);
    assert.match(
      orphan.stderr,
      /orphan\.rs must start with a rust-script shebang or be a reachable ordinary module/,
    );
    fs.rmSync(path.join(commands, "orphan.rs"));
    fs.rmSync(path.join(commands, "support/child.rs"));
    const missing = verify(fixture);
    assert.equal(missing.status, 1);
    assert.match(
      missing.stderr,
      /missing ordinary module: .*support\/child\.rs or .*support\/child\/mod\.rs/,
    );
    fs.writeFileSync(path.join(commands, "support/child.rs"), "");
    fs.mkdirSync(path.join(commands, "support/child"));
    fs.writeFileSync(path.join(commands, "support/child/mod.rs"), "");
    const ambiguous = verify(fixture);
    assert.equal(ambiguous.status, 1);
    assert.match(
      ambiguous.stderr,
      /ambiguous ordinary module: .*support\/child\.rs and .*support\/child\/mod\.rs/,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

function verify(cwd: string) {
  return spawnSync("rust-script", [path.join(root, "tools/commands/ci/verify-tool-layout.rs")], {
    cwd,
    encoding: "utf8",
  });
}

test("tool tree does not carry JavaScript command sources", () => {
  const javascriptTools = collectFiles(path.join(root, "tools"))
    .filter((file) => {
      const relative = normalize(path.relative(root, file));
      if (relative.startsWith("tools/commands/")) return false;
      if (relative.startsWith("tools/benchmarks/scripts/")) return false;
      if (relative.startsWith("tools/config/vite-plus/")) return false;
      if (relative.startsWith("tools/support/")) return false;
      if (relative.startsWith("tools/moon/.mooncakes/")) return false;
      return /\.(?:mjs|js|ts)$/.test(relative);
    })
    .map((file) => normalize(path.relative(root, file)))
    .sort();

  assert.deepEqual(javascriptTools, []);
  assert.equal(fs.existsSync(path.join(root, "tools", "rust")), false);
  assert.equal(fs.existsSync(path.join(root, "tools", "support", "legacy_command.rs")), false);
  assert.equal(fs.existsSync(path.join(root, "tools", "support", "tool_host.rs")), false);
});

function collectFiles(dir: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectFiles(absolute));
    } else if (entry.isFile()) {
      files.push(absolute);
    }
  }
  return files;
}

function normalize(filePath: string): string {
  return filePath.split(path.sep).join("/");
}

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf8");
}
