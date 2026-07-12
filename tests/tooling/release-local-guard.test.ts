import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { verifyLocalReleaseGuard } from "../../tools/github/release-local-guard.mjs";
import { writeFakeCommand } from "./support/fake-command.ts";

const HEAD_SHA = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SHA = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

interface GuardFixture {
  branch?: string;
  dirty?: boolean;
  ancestor?: boolean;
  headSha?: string;
  remoteSha?: string;
  localTagExists?: boolean;
  remoteTagExists?: boolean;
  hangs?: boolean;
}

function runGuard(
  options: GuardFixture = {},
  timeoutMs = 30_000,
): { error?: Error; gitLog: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-local-release-guard-"));
  const bin = path.join(root, "bin");
  const log = path.join(root, "git.log");
  fs.mkdirSync(bin, { recursive: true });
  fs.writeFileSync(log, "");
  const fixture = JSON.stringify(options);
  writeFakeCommand(
    bin,
    "git",
    [
      "const fs = require('node:fs');",
      `const fixture = ${fixture};`,
      "const args = process.argv.slice(2);",
      `fs.appendFileSync(${JSON.stringify(log)}, args.join(' ') + '\\n');`,
      "if (fixture.hangs) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1000);",
      "if (args[0] === 'branch') { console.log(fixture.branch ?? 'main'); process.exit(0); }",
      "if (args[0] === 'status') { if (fixture.dirty) console.log(' M Cargo.toml'); process.exit(0); }",
      "if (args[0] === 'fetch') process.exit(0);",
      "if (args[0] === 'merge-base') process.exit(fixture.ancestor === false ? 1 : 0);",
      "if (args[0] === 'rev-parse' && args.includes('--verify')) process.exit(fixture.localTagExists ? 0 : 1);",
      `if (args[0] === 'rev-parse') { console.log(args.at(-1) === 'HEAD' ? (fixture.headSha ?? '${HEAD_SHA}') : (fixture.remoteSha ?? '${HEAD_SHA}')); process.exit(0); }`,
      `if (args[0] === 'ls-remote') { if (fixture.remoteTagExists) console.log('${OTHER_SHA}\\t' + args.at(-1)); process.exit(fixture.remoteTagExists ? 0 : 2); }`,
      "process.exit(1);",
    ].join("\n"),
  );

  const originalPath = process.env.PATH;
  process.env.PATH = `${bin}${path.delimiter}${originalPath ?? ""}`;
  let error: Error | undefined;
  try {
    verifyLocalReleaseGuard("v0.290.1", root, timeoutMs);
  } catch (caught) {
    error = caught instanceof Error ? caught : new Error(String(caught));
  } finally {
    process.env.PATH = originalPath;
  }
  const gitLog = fs.readFileSync(log, "utf8");
  fs.rmSync(root, { recursive: true, force: true });
  return { error, gitLog };
}

test("local release guard accepts only the exact clean main tip", () => {
  const safe = runGuard();

  assert.equal(safe.error, undefined);
  assert.match(safe.gitLog, /^branch --show-current$/m);
  assert.match(safe.gitLog, /^fetch --quiet --no-tags origin /m);
  assert.match(safe.gitLog, /^ls-remote --exit-code --tags origin refs\/tags\/v0\.290\.1$/m);
  assert.doesNotMatch(safe.gitLog, /^(?:add|commit|tag|push)\b/m);
});

test("local release guard bounds stalled git commands", () => {
  const result = runGuard({ hangs: true }, 20);

  assert.match(result.error?.message ?? "", /git branch --show-current timed out after 20ms/);
});

test("local release guard rejects unsafe repository states", () => {
  const cases: Array<[GuardFixture, RegExp]> = [
    [{ branch: "feature/release" }, /local main branch/],
    [{ branch: "" }, /local main branch/],
    [{ dirty: true }, /uncommitted changes/],
    [{ ancestor: false }, /not reachable from the current origin\/main/],
    [{ remoteSha: OTHER_SHA }, /exactly match the current origin\/main/],
    [{ localTagExists: true }, /already exists locally/],
    [{ remoteTagExists: true }, /already exists and release tags are immutable/],
  ];

  for (const [fixture, expected] of cases) {
    const result = runGuard(fixture);
    assert.match(result.error?.message ?? "", expected);
    assert.doesNotMatch(result.gitLog, /^(?:add|commit|tag|push)\b/m);
  }
});
