import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { marketplaceQueryUrl } from "../../tools/support/release/marketplace_query.ts";
import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const scenarios = [
  {
    script: "publish_vscode_extension",
    metadataCommand: "curl",
    registry: "VS Code Marketplace",
  },
  {
    script: "publish_open_vsx_extension",
    metadataCommand: "curl",
    registry: "Open VSX",
  },
] as const;

for (const scenario of scenarios) {
  for (const publishExit of [0, 42]) {
    const outcome = publishExit === 0 ? "successful" : "failed";
    // vsce's zero exit is final; the Marketplace can take minutes to list it.
    const trustsExit = scenario.script === "publish_vscode_extension" && publishExit === 0;
    const verdict = trustsExit ? "accepts" : "rejects";
    test(`${scenario.script} ${verdict} an unconfirmed ${outcome} publish`, () => {
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
            "if (args[4] === 'show') process.exit(1);",
            "if (args[4] === 'create-namespace') process.exit(0);",
            "if (args[4] === 'publish') process.exit(Number(process.env.PUBLISH_EXIT));",
            "process.exit(1);",
          ].join("\n"),
        );
        writeFakeCommand(
          binDir,
          "curl",
          [
            "const fs = require('node:fs');",
            "const args = process.argv.slice(2);",
            "const calls = fs.existsSync(process.env.CALLS_PATH) ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, 'utf8')) : [];",
            "calls.push(['curl', ...args]);",
            "fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));",
            "process.exit(22); // curl --fail on HTTP 404",
          ].join("\n"),
        );

        const result = runReleaseCommand(scenario.script, [vsixPath, manifestPath], {
          env: {
            PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
            CALLS_PATH: callsPath,
            PUBLISH_EXIT: String(publishExit),
            PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
            PUBLISH_RESOLUTION_RETRY_DELAY: "0",
          },
        });

        assert.equal(result.status, trustsExit ? 0 : publishExit === 0 ? 1 : publishExit);
        if (publishExit === 0 && !trustsExit) {
          assert.match(result.stderr, new RegExp(`${scenario.registry} did not expose`));
        }
        const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
        const metadataCalls = calls.filter(
          (args) => args[4] === scenario.metadataCommand || args[0] === scenario.metadataCommand,
        );
        assert.equal(metadataCalls.length, trustsExit ? 1 : 2);
        for (const call of metadataCalls) {
          if (scenario.script === "publish_open_vsx_extension") {
            assert.deepEqual(call, [
              "curl",
              "--fail",
              "--silent",
              "--show-error",
              "https://open-vsx.org/api/ubugeeei/vize/universal/0.57.0",
            ]);
          } else {
            assert.equal(call.at(-1), marketplaceQueryUrl);
          }
        }
        assert.equal(calls.filter((args) => args[4] === "publish").length, 1);
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });
  }
}
