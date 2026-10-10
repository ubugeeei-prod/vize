import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";

export function installedMoon(repoRoot, env = process.env) {
  // The Actions shim exports its original MOON_HOME unconditionally. Resolve
  // the installed native executable before giving this fixture its own home.
  const homes = [
    env.MOON_HOME,
    env.RUNNER_TEMP && path.join(env.RUNNER_TEMP, "moonbit"),
    path.join(repoRoot, ".cache", "moonbit"),
  ].filter(Boolean);
  for (const home of homes) {
    const binary = path.resolve(repoRoot, home, "bin", "moon");
    if (fs.existsSync(binary)) return binary;
  }
  if (env.MOON_BIN && fs.existsSync(env.MOON_BIN)) {
    const prefix = fs.readFileSync(env.MOON_BIN).subarray(0, 2).toString();
    assert.notEqual(
      prefix,
      "#!",
      "fixture requires the native Moon executable, not a home-rewriting shim",
    );
    return path.resolve(repoRoot, env.MOON_BIN);
  }
  throw new Error("The pinned native Moon installation is required for the registry regression");
}

export function git(directory, args, env = process.env) {
  const result = spawnSync("git", ["-c", "commit.gpgsign=false", "-C", directory, ...args], {
    env,
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.stderr);
  assert.equal(result.status, 0, result.stderr);
  return result;
}

export function identity(directory, env) {
  return {
    head: git(directory, ["rev-parse", "HEAD"], env).stdout.trim(),
    tree: git(directory, ["rev-parse", "HEAD^{tree}"], env).stdout.trim(),
    status: git(directory, ["status", "--porcelain=v1", "--untracked-files=all"], env).stdout,
  };
}

export function registryFixture(root, env) {
  const origin = path.join(root, "origin");
  const home = path.join(root, "moonhome");
  const index = path.join(home, "registry", "index");
  fs.mkdirSync(origin);
  fs.mkdirSync(path.dirname(index), { recursive: true });
  git(origin, ["init", "--quiet", "--template=", "--initial-branch=main"], env);
  git(origin, ["config", "user.name", "Registry fixture"], env);
  git(origin, ["config", "user.email", "registry-fixture@example.invalid"], env);
  fs.writeFileSync(path.join(origin, "seed"), "seed\n");
  git(origin, ["add", "."], env);
  git(origin, ["commit", "--quiet", "-m", "seed"], env);
  const seed = identity(origin, env);
  git(root, ["clone", "--quiet", origin, index], env);
  const files = path.join(origin, "files");
  fs.mkdirSync(files);
  for (let i = 0; i < 10_000; i++) {
    fs.writeFileSync(path.join(files, `index-${String(i).padStart(5, "0")}.json`), "{}\n");
  }
  git(origin, ["add", "."], env);
  git(origin, ["commit", "--quiet", "-m", "large registry update"], env);
  fs.writeFileSync(
    path.join(home, "config.json"),
    JSON.stringify({ registry: origin, index: origin }),
  );
  return { origin, home, index, seed, target: identity(origin, env) };
}

export async function refuseSymbolsProxy() {
  let requests = 0;
  const server = http.createServer((_request, response) => {
    requests++;
    response.writeHead(503, { "Content-Length": "0" });
    response.end();
  });
  server.on("connect", (_request, socket) => {
    requests++;
    socket.end(
      "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const proxy = `http://127.0.0.1:${server.address().port}`;
  return {
    env: {
      HTTPS_PROXY: proxy,
      https_proxy: proxy,
      HTTP_PROXY: proxy,
      http_proxy: proxy,
      ALL_PROXY: "",
      all_proxy: "",
      NO_PROXY: "",
      no_proxy: "",
    },
    requests: () => requests,
    close: () =>
      new Promise((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  };
}

function ownedProcesses(group) {
  if (process.platform !== "linux") {
    const result = spawnSync("ps", ["-axo", "pid,ppid,pgid,state,comm"], { encoding: "utf8" });
    return result.stdout.split("\n").flatMap((line) => {
      const fields = line.trim().split(/\s+/, 5);
      return Number(fields[2]) === group
        ? [
            {
              pid: Number(fields[0]),
              ppid: Number(fields[1]),
              group,
              state: fields[3],
              binary: path.basename(fields[4]),
            },
          ]
        : [];
    });
  }
  const processes = [];
  for (const entry of fs.readdirSync("/proc")) {
    if (!/^\d+$/.test(entry)) continue;
    try {
      const stat = fs.readFileSync(`/proc/${entry}/stat`, "utf8");
      const boundary = stat.lastIndexOf(")");
      const fields = stat
        .slice(boundary + 1)
        .trim()
        .split(/\s+/);
      if (Number(fields[2]) !== group) continue;
      const links = {};
      for (const fd of fs.readdirSync(`/proc/${entry}/fd`)) {
        try {
          const target = fs.readlinkSync(`/proc/${entry}/fd/${fd}`);
          if (target.startsWith("pipe:[")) links[fd] = target;
        } catch {
          /* A child may close a descriptor during observation. */
        }
      }
      processes.push({
        pid: Number(entry),
        ppid: Number(fields[1]),
        group,
        state: fields[0],
        binary: stat.slice(stat.indexOf("(") + 1, boundary),
        wait: fs.readFileSync(`/proc/${entry}/wchan`, "utf8").trim(),
        pipes: links,
      });
    } catch {
      /* The process may have exited between directory reads. */
    }
  }
  return processes;
}

function killGroup(pid, signal) {
  try {
    process.kill(-pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

export function runOwned(command, args, { cwd, env, deadlineMs }) {
  return new Promise((resolve, reject) => {
    const started = performance.now();
    const child = spawn(command, args, {
      cwd,
      env,
      detached: true,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = [];
    const stderr = [];
    let expired = false;
    let snapshot = [];
    let escalation;
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    const deadline = setTimeout(() => {
      expired = true;
      snapshot = ownedProcesses(child.pid);
      killGroup(child.pid, "SIGTERM");
      escalation = setTimeout(() => killGroup(child.pid, "SIGKILL"), 250);
    }, deadlineMs);
    child.once("error", (error) => {
      clearTimeout(deadline);
      clearTimeout(escalation);
      reject(error);
    });
    child.once("close", (status, signal) => {
      clearTimeout(deadline);
      clearTimeout(escalation);
      // A surviving descendant belongs to this command's isolated group.
      if (child.pid != null) killGroup(child.pid, "SIGKILL");
      resolve({
        status,
        signal,
        expired,
        elapsedMs: performance.now() - started,
        snapshot,
        stdout: Buffer.concat(stdout).toString(),
        stderr: Buffer.concat(stderr).toString(),
      });
    });
  });
}
