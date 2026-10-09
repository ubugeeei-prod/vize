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
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const actionDir = fileURLToPath(
  new URL("../../../.github/actions/setup-rust-sticky-cache", import.meta.url),
);
const action = parse(readFileSync(join(actionDir, "action.yml"), "utf8"));

const backend = `import { appendFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join, resolve } from "node:path";
const request = JSON.parse(process.argv[2]);
const root = process.env.VIZE_FAKE_CACHE_ROOT;
const trace = row => appendFileSync(join(root, "trace.jsonl"), JSON.stringify({ ...row, pid: process.pid }) + "\\n");
const marker = join(root, "markers", request.name ?? "unknown");
const mountedMarker = join(root, "mounts", request.name ?? "unknown");
const storeFile = join(root, "stored-cache.json");
const entries = existsSync(storeFile) ? JSON.parse(readFileSync(storeFile,"utf8")) : [];
// Pinned @actions/cache 5.0.5 hashes path literals, compression and salt.
if (request.kind.startsWith("cache"))
  request.version=createHash("sha256").update([request.path,"zstd","1.0"].join("|")).digest("hex");
mkdirSync(join(root, "markers"), {recursive:true});
mkdirSync(join(root, "mounts"), {recursive:true});
if (request.kind === "sticky") {
  rmSync(mountedMarker, {force:true});
  if (!request.fail) {
    writeFileSync(mountedMarker,"mounted");
    writeFileSync(marker, "sticky clone");
    if (["target", "secondary"].includes(request.name)) {
      mkdirSync(request.path, { recursive: true });
      writeFileSync(join(request.path, "cache-artifact.txt"), "sticky clone");
    }
  }
  trace({...request, mounted: !request.fail});
} else if (request.kind === "cache") {
  const match = request.storedCache && entries.find(entry => entry.version===request.version &&
    (entry.key===request.key || (request.restorePrefix && entry.key.startsWith(request.restorePrefix))));
  if (request.storedCache) {
    request.hit=match ? match.key===request.key || "partial" : false;
    request.matchedKey=match?.key;
  }
  if (request.lookup) {
    if (readFileSync(marker, "utf8") !== "sticky clone") throw Error("mounted clone was overwritten");
  } else if (request.hit) {
    writeFileSync(marker, "Actions restore");
    if (match && ["target","secondary"].includes(request.name)) {
      mkdirSync(resolve(root,request.path), {recursive:true});
      writeFileSync(join(resolve(root,request.path),"cache-artifact.txt"),match.saved);
    }
  }
  trace({...request, hit: request.hit ?? false, restoredArtifact: request.lookup ? undefined : match?.saved});
} else if (request.kind === "workload") {
  const before = existsSync(marker) ? readFileSync(marker,"utf8") : null;
  const source = readFileSync(join(root,"source.txt"),"utf8");
  const artifact = createHash("sha256").update(source).digest("hex");
  writeFileSync(join(root,"artifact.txt"),artifact);
  mkdirSync(request.path, {recursive:true});
  writeFileSync(join(request.path,"cache-artifact.txt"),"compiled:"+artifact);
  if(readFileSync(join(root,"artifact.txt"),"utf8") !== artifact) throw Error("bad artifact");
  writeFileSync(marker,"compiled:"+artifact);
  trace({...request,before,artifact});
} else if (request.kind === "cache-post") {
  if (request.storedCache)
    request.hit=entries.some(entry=>entry.key===request.key && entry.version===request.version);
  if (request.hit !== true && !request.path) {
    trace({...request,saved:null,warning:"Input required and not supplied: path"});
  } else {
    const artifactPath = ["target", "secondary"].includes(request.name)
      ? join(resolve(root, request.path), "cache-artifact.txt") : marker;
    const saved = request.hit !== true && existsSync(artifactPath) ? readFileSync(artifactPath,"utf8") : null;
    if (request.storedCache && saved !== null) {
      entries.push({key:request.key,version:request.version,saved});
      writeFileSync(storeFile,JSON.stringify(entries));
    }
    trace({...request,saved});
  }
} else if (request.kind === "sticky-post") {
  if (!request.fail) {
    rmSync(marker,{force:true});
    rmSync(mountedMarker,{force:true});
  }
  trace(request);
} else if (request.kind === "probe") {
  trace(request);
  process.exit(existsSync(mountedMarker) ? 0 : 1);
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
      `#!${process.execPath}\nimport { spawnSync } from "node:child_process";\nimport { basename } from "node:path";\nconst path=process.argv[3]; const name=basename(path)==="registry"?"registry":basename(path)==="git"?"git":path===process.env.VIZE_FAKE_SECONDARY_PATH?"secondary":"target";\nconst result=spawnSync(process.execPath,[process.env.VIZE_FAKE_CACHE_ROOT+"/backend.mjs",JSON.stringify({kind:"probe",name,path})],{stdio:"inherit"});process.exit(result.status??1);\n`,
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

function value(expression, inputs, outputs, inner = false) {
  if (!inner && !expression.includes("${{")) return expression;
  const expr = expression.replace(/^\s*\$\{\{\s*|\s*\}\}\s*$/g, "").trim();
  for (const operator of [" || ", " && "]) {
    let depth = 0;
    for (let index = 0; index < expr.length; index++) {
      if (expr[index] === "(") depth++;
      if (expr[index] === ")") depth--;
      if (depth < 0) throw new Error("Unbalanced fixture expression");
      if (depth !== 0 || !expr.startsWith(operator, index)) continue;
      const left = value(expr.slice(0, index), inputs, outputs, true);
      const right = value(expr.slice(index + operator.length), inputs, outputs, true);
      return operator === " || " ? left || right : left && right;
    }
    if (depth !== 0) throw new Error("Unbalanced fixture expression");
  }
  if (expr.startsWith("(") && expr.endsWith(")"))
    return value(expr.slice(1, -1), inputs, outputs, true);
  const equality = expr.match(/^(.*) (==|!=) '([A-Za-z0-9._-]+)'$/);
  if (equality) {
    const equal =
      String(value(equality[1], inputs, outputs, true)).toLowerCase() === equality[3].toLowerCase();
    return equality[2] === "==" ? equal : !equal;
  }
  const step = expr.match(/^steps\.([\w-]+)\.outputs\.([\w-]+)$/);
  if (step) return outputs[step[1]]?.[step[2]] ?? "";
  const input = expr.match(/^inputs\.([\w-]+)$/);
  if (input) return inputs[input[1]] ?? "";
  if (expr === "runner.environment") return inputs.runner;
  if (expr === "runner.os") return inputs.os;
  if (expr === "runner.arch") return inputs.arch;
  throw new Error(`Unsupported fixture expression: ${expr}`);
}

export function executeCacheAction(
  fixture,
  context,
  {
    mountFailures = [],
    hit = false,
    nestedPost = false,
    storedCache = false,
    steps = action.runs.steps,
  } = {},
) {
  const { cwd } = fixture;
  const traceFile = join(cwd, "trace.jsonl");
  const traceStart = existsSync(traceFile)
    ? readFileSync(traceFile, "utf8").trim().split("\n").length
    : 0;
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
    VIZE_FAKE_SECONDARY_PATH: context.secondaryPath ? resolve(cwd, context.secondaryPath) : "",
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
  for (const step of steps) {
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
          : resolve(cwd, path) === outputs["cache-policy"]["secondary-path"]
            ? "secondary"
            : "target";
      if (step.uses.startsWith("useblacksmith/stickydisk@")) {
        const request = { name, key: args.key, path, fail: mountFailures.includes(name) };
        child({ kind: "sticky", ...request });
        posts.push({ kind: "sticky-post", ...request });
      } else if (
        step.uses.startsWith("actions/cache@") ||
        step.uses.startsWith("actions/cache/restore@")
      ) {
        const restoreOnly = step.uses.startsWith("actions/cache/restore@");
        const request = {
          name,
          key: args.key,
          path,
          lookup: args["lookup-only"] ?? false,
          hit,
          restoreOnly,
          storedCache,
          restorePrefix: args["restore-keys"],
        };
        child({ kind: "cache", ...request });
        if (!restoreOnly)
          posts.push({ kind: "cache-post", ...request, pathExpression: step.with.path });
      } else throw new Error(`Unexpected cache action: ${step.uses}`);
    }
  }
  if (status === 0) {
    child({ kind: "workload", name: "target", path: outputs["cache-policy"]["target-path"] });
    for (const request of posts.reverse()) {
      if (request.kind === "cache-post") {
        request.path = value(request.pathExpression, inputs, nestedPost ? {} : outputs);
        delete request.pathExpression;
      }
      child(request);
    }
  }
  const trace = existsSync(traceFile)
    ? readFileSync(traceFile, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line))
        .slice(traceStart)
    : [];
  return { status, outputs, trace };
}
