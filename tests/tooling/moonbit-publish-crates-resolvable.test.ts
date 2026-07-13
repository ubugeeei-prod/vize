import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { mkdtempSync, rmSync } from "node:fs";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("publish_crates accepts a failed publish once the crate resolves", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-resolvable-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
        "if (args[0] === 'publish' && args.at(-1) === 'vize_carton') process.exit(1);",
        "if (args[0] === 'publish' || args[0] === 'info') process.exit(0);",
        "process.exit(1);",
      ].join("\n"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      [
        "require('node:fs').writeSync(1, JSON.stringify({ errors: [{ detail: 'Not Found' }] }) + 'VIZE_HTTP_STATUS:404');",
        "process.exit(22);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_crates", [], {
      cwd: repoRoot,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CARGO_LOG: cargoLogPath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already resolvable despite a non-zero cargo publish exit/i);
    const logLines = fs.readFileSync(cargoLogPath, "utf8").trim().split("\n");
    assert.equal(logLines[0], "publish --locked -p vize_carton");
    assert.match(logLines[1] ?? "", /^info --registry crates-io vize_carton@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
