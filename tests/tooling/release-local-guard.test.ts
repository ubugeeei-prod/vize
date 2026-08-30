import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const HEAD_SHA = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SHA = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

interface GuardFixture {
  branch?: string;
  dirty?: boolean;
  ancestor?: boolean;
  headSha?: string;
  remoteSha?: string;
  parentLine?: string;
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
      `if (args[0] === 'rev-list') { console.log(fixture.parentLine ?? ((fixture.headSha ?? '${HEAD_SHA}') + ' ${OTHER_SHA}')); process.exit(0); }`,
      "if (args[0] === 'rev-parse' && args.includes('--verify')) process.exit(fixture.localTagExists ? 0 : 1);",
      `if (args[0] === 'rev-parse') { console.log(args.at(-1) === 'HEAD' ? (fixture.headSha ?? '${HEAD_SHA}') : (fixture.remoteSha ?? '${HEAD_SHA}')); process.exit(0); }`,
      `if (args[0] === 'ls-remote') { if (fixture.remoteTagExists) console.log('${OTHER_SHA}\\t' + args.at(-1)); process.exit(fixture.remoteTagExists ? 0 : 2); }`,
      "process.exit(1);",
    ].join("\n"),
  );

  const result = spawnSync(
    "rust-script",
    [path.join(repoRoot, "tools/commands/ci/github/release-local-guard.rs"), "v0.290.1"],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        VIZE_RELEASE_GUARD_GIT_TIMEOUT_MS: String(timeoutMs),
      },
    },
  );
  const error =
    result.status === 0 && result.error == null
      ? undefined
      : new Error(result.error?.message ?? result.stderr.trim());
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

test("local release guard accepts an exact merge commit at the main tip", () => {
  const mergeParent = "cccccccccccccccccccccccccccccccccccccccc";
  const safe = runGuard({ parentLine: `${HEAD_SHA} ${OTHER_SHA} ${mergeParent}` });

  assert.equal(safe.error, undefined);
  assert.doesNotMatch(safe.gitLog, /^rev-list\b/m);
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
