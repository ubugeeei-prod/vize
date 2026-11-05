import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { prepareWorkspace } from "../../tools/benchmarks/scripts/template-alias-paired-11822-custody.ts";
import { mapperCaptureAsset } from "../../tools/benchmarks/scripts/template-alias-paired-11822-capture.ts";

test("the actual mapper caller installs a byte-identical JS consumer runtime with lossless protocol capture", (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize mapper capture "));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const upstream = join(root, "upstream");
  const provider = join(root, "provider");
  const packageRoot = join(provider, "node_modules/vize");
  mkdirSync(join(upstream, "src"), { recursive: true });
  mkdirSync(packageRoot, { recursive: true });
  // Keep the production whole-corpus admission intact while using only fixture bytes.
  for (let index = 0; index < 2500; index++)
    writeFileSync(join(upstream, "src", `Component${index}.vue`), "<template />");
  writeFileSync(join(upstream, "tsconfig.base.json"), "{}");
  writeFileSync(
    join(upstream, "tsconfig.vize-content-mapper.json"),
    JSON.stringify({
      extends: "./tsconfig.base.json",
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    }),
  );
  writeFileSync(
    join(packageRoot, "package.json"),
    JSON.stringify({
      name: "vize",
      type: "module",
      files: [],
      typescript: {
        contentMapper: {
          exec: [
            "node",
            "-e",
            'if (process.argv.at(-1) !== "forwarded") process.exit(9); process.stdin.pipe(process.stdout);',
          ],
        },
      },
    }),
  );
  const workspace = join(root, "consumer");
  const capture = join(root, "protocol");
  const closure = prepareWorkspace(
    upstream,
    provider,
    workspace,
    {
      id: "candidate",
      package: packageRoot,
    },
    capture,
  );
  const manifest = JSON.parse(readFileSync(join(closure.packageRoot, "package.json"), "utf8")) as {
    typescript: { contentMapper: { exec: string[] } };
  };
  const [command, runtime, captureFlag, captureRoot, configuration] =
    manifest.typescript.contentMapper.exec;
  assert.equal(command, process.execPath);
  assert.equal(runtime, mapperCaptureAsset);
  assert.equal(captureFlag, "--capture");
  assert.equal(captureRoot, capture);
  assert.equal(configuration, join(capture, "command.json"));
  assert.deepEqual(JSON.parse(readFileSync(configuration, "utf8")), {
    command: [
      process.execPath,
      "-e",
      'if (process.argv.at(-1) !== "forwarded") process.exit(9); process.stdin.pipe(process.stdout);',
    ],
    packageRoot: closure.packageRoot,
  });
  const body = Buffer.from(JSON.stringify({ id: 1, text: "日本語", directives: [[0, 1]] }));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  const options = {
    cwd: workspace,
    env: { ...process.env, TEMPLATE_ALIAS_CAPTURE: capture },
    input: frame,
    timeout: 5000,
  };
  const run = spawnSync(
    command,
    [runtime, captureFlag, captureRoot, configuration, "forwarded"],
    options,
  );
  assert.equal(run.error, undefined);
  assert.equal(run.status, 0, run.stderr.toString());
  assert.deepEqual(run.stdout, frame);
  for (const prefix of ["in-", "out-"]) {
    const files = readdirSync(capture).filter((name) => name.startsWith(prefix));
    assert.equal(files.length, 1);
    assert.deepEqual(readFileSync(join(capture, files[0])), frame);
  }
  const typescriptRuntime = join(closure.packageRoot, "capture.ts");
  copyFileSync(runtime, typescriptRuntime);
  const blocked = spawnSync(command, [typescriptRuntime, "forwarded"], options);
  assert.notEqual(blocked.status, 0);
  assert.match(blocked.stderr.toString(), /ERR_UNSUPPORTED_NODE_MODULES_TYPE_STRIPPING/);
});
