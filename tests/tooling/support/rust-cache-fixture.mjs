import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const actionDir = fileURLToPath(
  new URL("../../../.github/actions/setup-rust-sticky-cache", import.meta.url),
);
const action = parse(readFileSync(join(actionDir, "action.yml"), "utf8"));

const backend = `import { appendFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";
const request = JSON.parse(process.argv[2]);
const root = process.env.VIZE_FAKE_CACHE_ROOT;
const trace = row => appendFileSync(join(root, "trace.jsonl"), JSON.stringify({ ...row, pid: process.pid }) + "\\n");
const marker = join(root, "markers", request.name ?? "unknown");
mkdirSync(join(root, "markers"), {recursive:true});
if (request.kind === "sticky") {
  if (!request.fail) writeFileSync(marker, "sticky clone");
  trace({...request, mounted: !request.fail});
} else if (request.kind === "cache") {
  if (request.lookup) {
    if (readFileSync(marker, "utf8") !== "sticky clone") throw Error("mounted clone was overwritten");
  } else if (request.hit) writeFileSync(marker, "Actions restore");
  trace({...request, hit: request.hit ?? false});
} else if (request.kind === "workload") {
  const before = existsSync(marker) ? readFileSync(marker,"utf8") : null;
  const source = readFileSync(join(root,"source.txt"),"utf8");
  const artifact = createHash("sha256").update(source).digest("hex");
  writeFileSync(join(root,"artifact.txt"),artifact);
  if(readFileSync(join(root,"artifact.txt"),"utf8") !== artifact) throw Error("bad artifact");
  writeFileSync(marker,"compiled:"+artifact);
  trace({...request,before,artifact});
} else if (request.kind === "cache-post") {
  const saved = request.hit !== true && existsSync(marker) ? readFileSync(marker,"utf8") : null;
  trace({...request,saved});
} else if (request.kind === "sticky-post") {
  if (!request.fail) rmSync(marker,{force:true});
  trace(request);
} else if (request.kind === "probe") {
  trace(request);
  process.exit(existsSync(marker) ? 0 : 1);
} else throw Error("unrecognized fake backend");
`;

function execute(program, args, cwd, env = process.env) {
  const result = spawnSync(program, args, { cwd, env, encoding: "utf8", shell: false });
  assert.equal(result.error, undefined);
  return result;
}

export function cacheFixture(callback) {
  const cwd = mkdtempSync(join(tmpdir(), "vize-cache-policy-"));
  try {
    writeFileSync(join(cwd, "Cargo.lock"), "frozen fixture lock\n");
    writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel="1.98.0"\n');
    writeFileSync(join(cwd, "source.txt"), "source that must be built on every cache outcome\n");
    const git = (...args) => {
      const result = execute("git", args, cwd);
      assert.equal(result.status, 0, result.stderr);
      return result.stdout.trim();
    };
    git("init", "--quiet");
    git("add", "Cargo.lock", "rust-toolchain.toml");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.test",
      "commit",
      "--quiet",
      "-m",
      "fixture",
    );
    const sha = git("rev-parse", "HEAD");
    mkdirSync(join(cwd, "bin"));
    writeFileSync(join(cwd, "backend.mjs"), backend);
    const probe = join(cwd, "bin/mountpoint");
    writeFileSync(
      probe,
      `#!${process.execPath}\nimport { spawnSync } from "node:child_process";\nimport { basename } from "node:path";\nconst path=process.argv[3]; const name=basename(path)==="registry"?"registry":basename(path)==="git"?"git":basename(path)==="secondary"?"secondary":"target";\nconst result=spawnSync(process.execPath,[process.env.VIZE_FAKE_CACHE_ROOT+"/backend.mjs",JSON.stringify({kind:"probe",name,path})],{stdio:"inherit"});process.exit(result.status??1);\n`,
    );
    chmodSync(probe, 0o755);
    const context = (eventName = "push", ref = "refs/heads/main") => ({
      eventName,
      ref,
      repository: "example/repo",
      sourceSha: sha,
      checkoutSha: sha,
      runnerEnvironment: "self-hosted",
      runnerOs: "Linux",
      runnerArch: "X64",
      role: "test-scripts",
      suffix: "Linux-X64",
      targetPath: "target",
      event: {
        repository: { full_name: "example/repo", default_branch: "main" },
        ref,
        deleted: false,
        pull_request: { number: 1 },
        merge_group: { head_sha: sha },
      },
    });
    callback({ cwd, sha, git, context });
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
}

function value(expression, inputs, outputs) {
  const expr = expression.replace(/^\s*\$\{\{\s*|\s*\}\}\s*$/g, "").trim();
  if (expr.includes(" && "))
    return expr.split(" && ").every((part) => value(part, inputs, outputs));
  const equality = expr.match(/^(.*) == 'true'$/);
  if (equality) return value(equality[1], inputs, outputs) === "true";
  const step = expr.match(/^steps\.([\w-]+)\.outputs\.([\w-]+)$/);
  if (step) return outputs[step[1]]?.[step[2]] ?? "";
  const input = expr.match(/^inputs\.([\w-]+)$/);
  if (input) return inputs[input[1]] ?? "";
  if (expr === "runner.environment") return inputs.runner;
  if (expr === "runner.os") return inputs.os;
  if (expr === "runner.arch") return inputs.arch;
  if (!expression.includes("${{")) return expression;
  throw new Error(`Unsupported fixture expression: ${expr}`);
}

export function executeCacheAction(fixture, context, { mountFailures = [], hit = false } = {}) {
  const { cwd } = fixture;
  const inputs = {
    key: context.role,
    "cache-key-suffix": context.suffix,
    "target-path": context.targetPath,
    "secondary-key": context.secondaryRole ?? "",
    "secondary-cache-key-suffix": context.secondarySuffix ?? "",
    "secondary-target-path": context.secondaryPath ?? "",
    runner: context.runnerEnvironment,
    os: context.runnerOs,
    arch: context.runnerArch,
  };
  writeFileSync(join(cwd, "event.json"), JSON.stringify(context.event));
  const outputs = {};
  const env = {
    ...process.env,
    PATH: `${join(cwd, "bin")}:${dirname(process.execPath)}:${process.env.PATH}`,
    GITHUB_ACTION_PATH: actionDir,
    GITHUB_EVENT_PATH: join(cwd, "event.json"),
    GITHUB_EVENT_NAME: context.eventName,
    GITHUB_REF: context.ref,
    GITHUB_SHA: context.sourceSha,
    GITHUB_REPOSITORY: context.repository,
    GITHUB_WORKSPACE: cwd,
    VIZE_CACHE_SOURCE_ROOT: cwd,
    VIZE_FAKE_CACHE_ROOT: cwd,
  };
  const child = (request) => {
    const result = execute(
      process.execPath,
      [join(cwd, "backend.mjs"), JSON.stringify(request)],
      cwd,
      env,
    );
    assert.equal(result.status, 0, result.stderr);
  };
  let status = 0;
  const posts = [];
  for (const step of action.runs.steps) {
    if (step.if && !value(step.if, inputs, outputs)) continue;
    if (step.run) {
      const file = join(cwd, `output-${step.id}.txt`);
      writeFileSync(file, "");
      const stepEnv = Object.fromEntries(
        Object.entries(step.env ?? {}).map(([key, expr]) => [
          key,
          String(value(expr, inputs, outputs)),
        ]),
      );
      const result = execute("bash", ["-e", "-c", step.run], cwd, {
        ...env,
        ...stepEnv,
        GITHUB_OUTPUT: file,
      });
      if (result.status !== 0) {
        status = result.status ?? 1;
        break;
      }
      outputs[step.id] = Object.fromEntries(
        readFileSync(file, "utf8")
          .trim()
          .split("\n")
          .filter(Boolean)
          .map((line) => {
            const split = line.indexOf("=");
            return [line.slice(0, split), line.slice(split + 1)];
          }),
      );
    } else {
      const args = Object.fromEntries(
        Object.entries(step.with).map(([key, expr]) => [key, value(expr, inputs, outputs)]),
      );
      const path = String(args.path);
      const name = path.endsWith("registry")
        ? "registry"
        : path.endsWith("git")
          ? "git"
          : path.endsWith("secondary")
            ? "secondary"
            : "target";
      if (step.uses.startsWith("useblacksmith/stickydisk@")) {
        const request = { name, key: args.key, path, fail: mountFailures.includes(name) };
        child({ kind: "sticky", ...request });
        posts.push({ kind: "sticky-post", ...request });
      } else if (step.uses.startsWith("actions/cache@")) {
        const request = { name, key: args.key, path, lookup: args["lookup-only"], hit };
        child({ kind: "cache", ...request });
        posts.push({ kind: "cache-post", ...request });
      } else throw new Error(`Unexpected cache action: ${step.uses}`);
    }
  }
  if (status === 0) {
    child({ kind: "workload", name: "target" });
    for (const request of posts.reverse()) child(request);
  }
  const traceFile = join(cwd, "trace.jsonl");
  const trace = existsSync(traceFile)
    ? readFileSync(traceFile, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line))
    : [];
  return { status, outputs, trace };
}
