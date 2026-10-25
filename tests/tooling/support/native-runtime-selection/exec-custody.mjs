import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { sha256 } from "./providers.mjs";

export function parseExecTrace(text) {
  const rows = [];
  for (const [index, line] of text.split("\n").entries()) {
    if (!line.includes("execve(")) continue;
    const match = /^execve\(("(?:[^"\\]|\\.)*"), (\[.*\]), .*\)\s+= (.*)$/u.exec(line);
    assert.ok(match, `complete execve framing required at line ${index + 1}`);
    const executable = JSON.parse(match[1]),
      argv = JSON.parse(match[2]);
    assert.equal(typeof executable, "string");
    assert.ok(Array.isArray(argv) && argv.every((x) => typeof x === "string"));
    rows.push({
      line: index + 1,
      raw: line,
      executable,
      argv,
      success: match[3] === "0",
      result: match[3],
    });
  }
  return rows;
}
function fileHash(file) {
  const hash = createHash("sha256"),
    fd = fs.openSync(file, "r"),
    buffer = Buffer.alloc(65536);
  try {
    let n;
    while ((n = fs.readSync(fd, buffer, 0, buffer.length, null)))
      hash.update(buffer.subarray(0, n));
  } finally {
    fs.closeSync(fd);
  }
  return hash.digest("hex");
}
export function runStracedCli({ strace, cli, args, cwd, env, directory, expectedExecutable }) {
  fs.mkdirSync(directory, { recursive: true });
  const prefix = path.join(directory, "execve"),
    stdout = path.join(directory, "stdout.bin"),
    stderr = path.join(directory, "stderr.bin");
  const out = fs.openSync(stdout, "w"),
    err = fs.openSync(stderr, "w");
  let result;
  const argv = [
    "--kill-on-exit",
    "-ff",
    "-s",
    "65536",
    "-e",
    "trace=execve",
    "-o",
    prefix,
    cli,
    ...args,
  ];
  try {
    result = spawnSync(strace, argv, { cwd, env, stdio: ["ignore", out, err], timeout: 45000 });
  } finally {
    fs.closeSync(out);
    fs.closeSync(err);
  }
  const rawFiles = fs
    .readdirSync(directory)
    .filter((x) => x.startsWith("execve."))
    .sort()
    .map((file) => ({
      file,
      bytes: fs.statSync(path.join(directory, file)).size,
      sha256: fileHash(path.join(directory, file)),
    }));
  const streams = Object.fromEntries(
    [
      ["stdout", stdout],
      ["stderr", stderr],
    ].map(([name, file]) => {
      const size = fs.statSync(file).size;
      return [
        name,
        {
          file,
          bytes: size,
          sha256: fileHash(file),
          text: size <= 16 * 1024 * 1024 ? fs.readFileSync(file, "utf8") : null,
        },
      ];
    }),
  );
  const receipt = {
    argv: [strace, ...argv],
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    streams,
    rawFiles,
    selectedExecutions: [],
    validation: "pending",
    meaning:
      "Source CLI+SDK execution selection only; strace observer cost is present; no performance/API/lifecycle qualification.",
  };
  const save = () =>
    fs.writeFileSync(path.join(directory, "process.json"), JSON.stringify(receipt, null, 2) + "\n");
  save();
  try {
    assert.equal(receipt.error, null);
    assert.equal(receipt.signal, null);
    assert.ok(rawFiles.length > 0, "actual execve traces are mandatory");
    for (const stream of Object.values(streams))
      assert.ok(stream.bytes <= 16 * 1024 * 1024, "existing per-stream observation bound");
    for (const file of rawFiles) {
      assert.ok(file.bytes <= 16 * 1024 * 1024, "bounded complete execve observation");
      file.events = parseExecTrace(fs.readFileSync(path.join(directory, file.file), "utf8"));
    }
    const successful = rawFiles.flatMap((file) => file.events).filter((x) => x.success);
    assert.ok(
      successful.some((x) => fs.realpathSync(x.executable) === cli),
      "actual source CLI exec required",
    );
    const children = successful.filter((x) => fs.realpathSync(x.executable) !== cli);
    receipt.selectedExecutions = children.map((child) => ({
      ...child,
      physicalPath: fs.realpathSync(child.executable),
      sha256: sha256(fs.readFileSync(child.executable)),
    }));
    save();
    if (expectedExecutable) {
      assert.ok(children.length > 0, "actual native compiler exec required");
      for (const child of receipt.selectedExecutions) {
        assert.equal(
          child.physicalPath,
          expectedExecutable.path,
          "only selected physical SDK may execute",
        );
        assert.equal(child.sha256, expectedExecutable.sha256);
        assert.ok(
          !child.argv.includes("--api") && !child.argv.includes("--lsp"),
          "this observation grants CLI-only capability",
        );
      }
    } else
      assert.deepEqual(
        children,
        [],
        "format/missing-runtime controls must never execute a native compiler",
      );
    receipt.validation = "PASS";
    save();
    return receipt;
  } catch (error) {
    receipt.validation = "FAILED";
    receipt.validationError = { message: String(error), stack: error?.stack ?? null };
    save();
    throw error;
  }
}
