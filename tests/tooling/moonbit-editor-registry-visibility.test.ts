import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const scenarios = [
  {
    script: "publish_vscode_extension",
    metadataCommand: "show",
    registry: "VS Code Marketplace",
  },
  {
    script: "publish_open_vsx_extension",
    metadataCommand: "get",
    registry: "Open VSX",
  },
] as const;

for (const scenario of scenarios) {
  for (const publishExit of [0, 42]) {
    const outcome = publishExit === 0 ? "successful" : "failed";
    test(`${scenario.script} rejects an unconfirmed ${outcome} publish`, () => {
      const tempDir = mkdtempSync(path.join(tmpdir(), "editor-registry-visibility-"));
      const binDir = path.join(tempDir, "bin");
      const manifestPath = path.join(tempDir, "package.json");
      const vsixPath = path.join(tempDir, "vize.vsix");
      const callsPath = path.join(tempDir, "calls.json");

      try {
        fs.mkdirSync(binDir, { recursive: true });
        writeFileSync(
          manifestPath,
          `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" })}\n`,
        );
        writeFileSync(vsixPath, "vsix");
        writeFakeCommand(
          binDir,
          "vp",
          [
            "const fs = require('node:fs');",
            "const args = process.argv.slice(2);",
            "const calls = fs.existsSync(process.env.CALLS_PATH) ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, 'utf8')) : [];",
            "calls.push(args);",
            "fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));",
            "if (args[4] === 'show' || args[4] === 'get') process.exit(1);",
            "if (args[4] === 'create-namespace') process.exit(0);",
            "if (args[4] === 'publish') process.exit(Number(process.env.PUBLISH_EXIT));",
            "process.exit(1);",
          ].join("\n"),
        );

        const result = runMoonScript(scenario.script, [vsixPath, manifestPath], {
          env: {
            PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
            CALLS_PATH: callsPath,
            PUBLISH_EXIT: String(publishExit),
            PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
            PUBLISH_RESOLUTION_RETRY_DELAY: "0",
          },
        });

        assert.equal(result.status, publishExit === 0 ? 1 : publishExit);
        if (publishExit === 0) {
          assert.match(result.stderr, new RegExp(`${scenario.registry} did not expose`));
        }
        const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
        assert.equal(calls.filter((args) => args[4] === scenario.metadataCommand).length, 2);
        assert.equal(calls.filter((args) => args[4] === "publish").length, 1);
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });
  }
}
