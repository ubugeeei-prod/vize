import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "commands", "fixtures", "tool-matrix-report.rs");

function runToolMatrix(args: string[]) {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-matrix-"));
  const result = spawnSync("rust-script", [toolPath, ...args, "--output-dir", outputDir], {
    cwd: root,
    encoding: "utf8",
  });
  return { outputDir, result };
}

function writeFakeVize(directory: string, body: string) {
  const executable = path.join(directory, "fake-vize.mjs");
  fs.writeFileSync(executable, `#!/usr/bin/env node\n${body}\n`);
  fs.chmodSync(executable, 0o755);
  return executable;
}

function toolMatrixEvents(stderr: string) {
  return stderr
    .split("\n")
    .filter((line) => line.startsWith("[tool-matrix] "))
    .map((line) => {
      const [, event, ...fieldParts] = line.split(" ");
      const fields = new Map<string, string>();
      for (const part of fieldParts) {
        const separator = part.indexOf("=");
        assert.notEqual(separator, -1, `invalid progress field: ${part}`);
        fields.set(part.slice(0, separator), part.slice(separator + 1));
      }
      return { event, fields };
    });
}

function requireEvent(events: ReturnType<typeof toolMatrixEvents>, event: string, afterIndex = -1) {
  const index = events.findIndex((entry, candidateIndex) => {
    return candidateIndex > afterIndex && entry.event === event;
  });
  assert.notEqual(
    index,
    -1,
    `missing ${event} event after index ${afterIndex}; saw ${events
      .map((entry) => entry.event)
      .join(", ")}`,
  );
  return { index, fields: events[index].fields };
}

test("fixture tool matrix emits progress while a tool invocation is running", () => {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-progress-"));
  const executable = writeFakeVize(
    fakeDir,
    `if (process.argv[2] === "--version") process.exit(0);
setTimeout(() => {
  process.stdout.write("not json");
  process.stderr.write("synthetic failure");
  process.exit(2);
}, 80);`,
  );
  const { outputDir, result } = runToolMatrix([
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler",
    "--vize-bin",
    executable,
    "--timeout-ms",
    "5000",
    "--heartbeat-ms",
    "20",
  ]);
  try {
    assert.equal(result.status, 1);
    const events = toolMatrixEvents(result.stderr);
    const start = requireEvent(events, "start");
    const heartbeat = requireEvent(events, "still-running", start.index);
    const finish = requireEvent(events, "finish", heartbeat.index);

    for (const { fields } of [start, heartbeat, finish]) {
      assert.equal(fields.get("projectId"), "vue-vben-admin");
      assert.equal(fields.get("tool"), "compiler");
    }
    assert.equal(start.fields.get("timeoutMs"), "5000");
    assert.match(heartbeat.fields.get("elapsedMs") ?? "", /^\d+$/);
    assert.match(finish.fields.get("elapsedMs") ?? "", /^\d+$/);
    assert.equal(finish.fields.get("status"), "2");
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

test("fixture tool matrix bounds timeout when child descendants inherit stdio", () => {
  if (process.platform === "win32") return;

  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-tool-timeout-tree-"));
  const executable = writeFakeVize(
    fakeDir,
    `import { spawn } from "node:child_process";
if (process.argv[2] === "--version") process.exit(0);
spawn(process.execPath, ["-e", "setTimeout(() => {}, 4000)"], { stdio: "inherit" });
setInterval(() => {}, 1000);`,
  );
  const startedAt = Date.now();
  const { outputDir, result } = runToolMatrix([
    "--project",
    "vue-vben-admin",
    "--tool",
    "compiler",
    "--vize-bin",
    executable,
    "--timeout-ms",
    "80",
    "--heartbeat-ms",
    "20",
  ]);
  try {
    assert.equal(result.status, 1);
    // This must stay below the old 5s force-kill escalation while allowing
    // Rust Script startup and cache lookup overhead.
    assert.ok(
      Date.now() - startedAt < 4_000,
      "timeout handling must not wait for descendants that inherited stdio",
    );
    const report = JSON.parse(fs.readFileSync(path.join(outputDir, "summary.json"), "utf8"));
    const rawPath = path.resolve(root, report.projects[0].runs[0].outputPath);
    const raw = JSON.parse(fs.readFileSync(rawPath, "utf8"));
    assert.equal(raw.spawnError, "spawn timed out after 80ms");
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});
