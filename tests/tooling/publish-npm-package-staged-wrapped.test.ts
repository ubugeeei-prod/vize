import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const fixturePath = path.join(
  repoRoot,
  "tests/_fixtures/tooling/npm-publish-staged/wrapped-409.stderr.txt",
);
const original = fs.readFileSync(fixturePath);
assert.equal(original.length, 224);
assert.equal(
  createHash("sha256").update(original).digest("hex"),
  "d77a1a7b8aaa18b730238beedb4e8536f6f8443c95925697908fa472b122da87",
);
const wrapped = original.toString();
assert(!wrapped.includes("Cannot publish over previously staged version"));

type Case = {
  name: string;
  detail: string;
  stream?: "stdout";
  generic?: boolean;
  hidden?: boolean;
  wrongTag?: boolean;
};
const cases: Case[] = [
  { name: "whole real wrapped409 LF", detail: wrapped },
  { name: "whole real wrapped409 CRLF", detail: wrapped.replaceAll("\n", "\r\n") },
  {
    name: "wrapping earlier inside the staged phrase",
    detail: wrapped.replace(
      "Cannot publish over previously staged\n  │ version",
      "Cannot publish\n\t│ over previously\n  │ staged\n  │ version",
    ),
  },
  { name: "the same whole diagnostic on stdout", detail: wrapped, stream: "stdout" },
  {
    name: "an unrelated409 still takes ordinary retry",
    detail: wrapped.replace("previously staged", "previously published"),
    generic: true,
  },
  {
    name: "authorization failure still takes ordinary retry",
    detail:
      "Error: ERR_PNPM_FAILED_TO_PUBLISH\n\n" +
      "  × Failed to publish package @vizejs/native-darwin-arm64@0.433.0 (status403\n" +
      '  │ Forbidden): {"error":"No permission to publish this package."}\n',
    generic: true,
  },
  {
    name: "hidden staged version exhausts only bounded visibility polls",
    detail: wrapped,
    hidden: true,
  },
  {
    name: "visible staged version with wrong latest tag still fails",
    detail: wrapped,
    wrongTag: true,
  },
];

for (const row of cases) {
  test(`publish_npm_package: ${row.name}`, () => {
    const directory = fs.mkdtempSync(path.join(tmpdir(), "moonbit-staged-wrap-"));
    const packageDir = path.join(directory, "pkg");
    const binDir = path.join(directory, "bin");
    const statePath = path.join(directory, "state.json");
    const detailPath = path.join(directory, "detail.txt");
    try {
      fs.mkdirSync(packageDir);
      fs.mkdirSync(binDir);
      fs.writeFileSync(
        path.join(packageDir, "package.json"),
        JSON.stringify({ name: "@vizejs/native-darwin-arm64", version: "0.433.0" }) + "\n",
      );
      fs.writeFileSync(detailPath, row.detail);
      fs.writeFileSync(
        statePath,
        JSON.stringify({ publishCalls: 0, versionChecks: 0, tagChecks: 0, invocations: [] }),
      );
      writeFakeCommand(
        binDir,
        "vp",
        [
          "const fs = require('node:fs');",
          "const state = JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'));",
          "state.publishCalls++;",
          "state.invocations.push({ arguments: process.argv.slice(2), cwd: process.cwd() });",
          "if (process.env.WRAP_GENERIC === '1' && state.publishCalls === 3) state.published = true;",
          "fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
          "if (state.published) process.exit(0);",
          "process[process.env.WRAP_STREAM].write(fs.readFileSync(process.env.WRAP_DETAIL));",
          "process.exit(1);",
        ].join("\n"),
      );
      writeFakeCommand(
        binDir,
        "curl",
        [
          "const fs = require('node:fs');",
          "const state = JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'));",
          "const url = new URL(process.argv.at(-1));",
          "const exact = url.pathname.endsWith('/0.433.0');",
          "if (exact) state.versionChecks++; else state.tagChecks++;",
          "fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
          "const visible = process.env.WRAP_HIDDEN !== '1' && (state.published",
          "  || (state.publishCalls === 1 && state.versionChecks >= 3));",
          "if (!visible) { process.stdout.write('{}VIZE_HTTP_STATUS:404'); process.exit(0); }",
          "const name = '@vizejs/native-darwin-arm64';",
          "const tag = process.env.WRAP_WRONG_TAG === '1' ? '0.432.0' : '0.433.0';",
          "const body = exact ? { name, version: '0.433.0' } : { name, 'dist-tags': { latest: tag } };",
          "process.stdout.write(JSON.stringify(body) + 'VIZE_HTTP_STATUS:200');",
        ].join("\n"),
      );
      const result = runMoonScript("publish_npm_package", [packageDir, "--provenance"], {
        env: {
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          VP_BIN: path.join(binDir, "vp"),
          VP_STATE_PATH: statePath,
          WRAP_DETAIL: detailPath,
          WRAP_STREAM: row.stream ?? "stderr",
          WRAP_GENERIC: row.generic ? "1" : "0",
          WRAP_HIDDEN: row.hidden ? "1" : "0",
          WRAP_WRONG_TAG: row.wrongTag ? "1" : "0",
          NPM_TAG: "latest",
          PUBLISH_RETRY_LIMIT: "3",
          PUBLISH_RETRY_DELAY: "1",
          PUBLISH_RESOLUTION_RETRY_LIMIT: "3",
          PUBLISH_RESOLUTION_RETRY_DELAY: "1",
        },
      });
      const boundedFailure = row.hidden || row.wrongTag;
      assert.equal(result.status, boundedFailure ? 1 : 0, `${result.stderr}\n${result.stdout}`);
      const state = JSON.parse(fs.readFileSync(statePath, "utf8")) as {
        publishCalls: number;
        versionChecks: number;
        tagChecks: number;
        invocations: { arguments: string[]; cwd: string }[];
      };
      assert.equal(state.publishCalls, row.generic ? 3 : 1);
      assert.equal(state.versionChecks, boundedFailure ? 5 : row.generic ? 4 : 3);
      assert.equal(state.tagChecks, boundedFailure ? 3 : 1);
      assert.deepEqual(
        state.invocations,
        Array.from({ length: state.publishCalls }, () => ({
          arguments: [
            "pm",
            "publish",
            "--access",
            "public",
            "--no-git-checks",
            "--tag",
            "latest",
            "--",
            "--provenance",
          ],
          cwd: fs.realpathSync(packageDir),
        })),
      );
      const timeout = boundedFailure
        ? "Timed out waiting for @vizejs/native-darwin-arm64@0.433.0 to appear in npm with dist-tag latest.\n"
        : "";
      const diagnostic = (row.detail.trim() + "\n").repeat(row.generic ? 2 : 1);
      assert.equal(result.stderr, (row.stream ? "" : diagnostic) + timeout);
      if (row.stream) assert(result.stdout.includes(row.detail.trim() + "\n"));
      assert.equal(
        (result.stdout.match(/Publish attempt failed/g) ?? []).length,
        row.generic ? 2 : 0,
      );
      assert.equal(
        (result.stdout.match(/is staged in npm but not yet visible/g) ?? []).length,
        row.generic ? 0 : 1,
      );
      assert.equal(
        (result.stdout.match(/is visible in npm with dist-tag latest/g) ?? []).length,
        boundedFailure ? 0 : 1,
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
