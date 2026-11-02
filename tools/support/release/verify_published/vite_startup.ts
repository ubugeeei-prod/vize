import {
  authenticateCapsule,
  contract,
  hash,
  image,
  json,
  managerRoute,
  object,
  packageAuthenticator,
  packageRoot,
  safePath,
  target,
  type ObjectValue,
} from "./vite_startup_authority.ts";
export { authenticateCapsule, contract, hash } from "./vite_startup_authority.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { boundedBytes } from "../release/quota/http.ts";
import { readReleaseOwnership } from "../release/quota_ownership.ts";
import { cleanEnvironment } from "../human_reports_11252_11253_public/capture.ts";

type File = { path: string; source: string };
export type Result = {
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
};
const controls = object(contract.controls);
export function childEnvironment(input: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  const clean = cleanEnvironment(undefined, input);
  for (const key of [
    "NODE_OPTIONS",
    "NODE_PATH",
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  ])
    assert(!Object.hasOwn(input, key), "unowned runtime override presence: " + key);
  return Object.fromEntries(
    Object.entries(clean).filter(([key]) => !/TOKEN|SECRET|PASSWORD|CREDENTIAL/i.test(key)),
  );
}
export function spawnOptions(cwd: string, env: NodeJS.ProcessEnv) {
  return { cwd, env: childEnvironment(env), timeout: 300_000, maxBuffer: 8 * 1024 * 1024 };
}
export function collect(
  command: string,
  argv: string[],
  cwd: string,
  env: NodeJS.ProcessEnv,
  retain: (row: object, stdout: Buffer, stderr: Buffer) => void,
  spawn: typeof spawnSync = spawnSync,
): Result {
  const started = new Date().toISOString(),
    result = spawn(command, argv, spawnOptions(cwd, env));
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  const row = {
    argv: [command, ...argv],
    cwd,
    started,
    completed: new Date().toISOString(),
    status: result.status,
    signal: result.signal,
    error: result.error?.name ?? null,
    pid: result.pid,
    errorCode: result.error && "code" in result.error ? String(result.error.code) : null,
    stdout: { bytes: stdout.length, sha256: hash(stdout) },
    stderr: { bytes: stderr.length, sha256: hash(stderr) },
  };
  retain(row, stdout, stderr);
  return {
    status: result.status,
    signal: result.signal,
    error: result.error?.name ?? null,
    stdout: new TextDecoder("utf-8", { fatal: true }).decode(stdout),
    stderr: new TextDecoder("utf-8", { fatal: true }).decode(stderr),
  };
}
export function originalFiles(issue: number): File[] {
  const original = (contract.originals as ObjectValue[]).find((v) => v.issue === issue);
  assert(original);
  const files = structuredClone(original.files) as File[];
  assert.equal(files.length, 7);
  const manifest = files.find((f) => f.path === "package.json")!;
  const before = issue === 11375 ? "0.640.0" : "0.641.0";
  const count = manifest.source.split(before).length - 1;
  assert.equal(count, issue === 11375 ? 1 : 2);
  manifest.source = manifest.source.replaceAll(before, String(target.version));
  return files;
}
export function stateFiles(issue: number, state: string): File[] {
  const files = originalFiles(issue),
    get = (path: string) => {
      const f = files.find((v) => v.path === path);
      assert(f);
      return f;
    };
  if (issue === 11375) {
    assert(
      ["original-vize", "original-official", "import-invalid", "import-repaired"].includes(state),
    );
    if (state.startsWith("import-"))
      get("src/main.ts").source = String(controls.importPrefix) + get("src/main.ts").source;
    if (state === "import-repaired")
      get("src/Placeholder.vue").source = String(controls.repairedPlaceholder);
  } else {
    assert.equal(issue, 11376);
    assert(
      [
        "original",
        "sentinel-canonical",
        "sentinel-ignore-override",
        "sentinel-scan-override",
      ].includes(state),
    );
    if (state !== "original")
      for (const p of ["src/Ignored.vue", "other/Outside.vue"])
        get(p).source = String(controls.missingSource);
    if (state === "sentinel-ignore-override")
      get("vite.config.ts").source = get("vite.config.ts").source.replace(
        '  ignores: ["src/Ignored.vue"],',
        '  ignores: ["src/Ignored.vue"],\n  vite: { ignorePatterns: [] },',
      );
    if (state === "sentinel-scan-override")
      get("vite.config.ts").source = get("vite.config.ts").source.replace(
        '  ignores: ["src/Ignored.vue"],',
        '  ignores: ["src/Ignored.vue"],\n  vite: { scanPatterns: ["other/**/*.vue"] },',
      );
  }
  return files;
}
export function argumentsFor(issue: number, state: string): string[] {
  stateFiles(issue, state);
  return issue === 11375
    ? [
        "exec",
        "vite",
        "build",
        "-c",
        state === "original-official" ? "vite.config.pluginvue.ts" : "vite.config.vize.ts",
      ]
    : ["exec", "vp", "build"];
}
export function verdict(issue: number, state: string, result: Result, cwd: string) {
  assert.equal(result.signal, null);
  assert.equal(result.error, null);
  const negative = state === "import-invalid" || state.endsWith("-override");
  if (!negative) {
    assert.equal(result.status, 0);
    return;
  }
  assert.equal(typeof result.status, "number");
  assert.notEqual(result.status, 0);
  const whole = result.stdout + result.stderr;
  if (issue === 11375) {
    // The startup warning alone is insufficient: require the actual Vite build error.
    const failure = whole.slice(whole.lastIndexOf("error during build:"));
    assert(whole.includes("error during build:"));
    assert(failure.includes(String(controls.missingBlock)));
    assert(failure.includes("Placeholder.vue"));
  } else {
    const selected = state === "sentinel-ignore-override" ? "src/Ignored.vue" : "other/Outside.vue";
    const expected =
      String(controls.missingSourcePrefix) +
      resolve(cwd, dirname(selected), "missing.html") +
      ") in " +
      resolve(cwd, selected);
    assert(whole.includes(expected));
  }
}
export function verifyInputs(root: string, files: File[]) {
  for (const file of files) {
    const path = safePath(root, file.path);
    assert(lstatSync(path).isFile());
    assert.equal(readFileSync(path, "utf8"), file.source);
  }
  for (const path of ["src/missing.html", "other/missing.html"])
    assert(!existsSync(join(root, path)), "registered missing-src target must stay absent");
}
export async function run() {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.version, object(contract.adaptation).node);
  const [version, blob, managerDirectory] = process.argv.slice(2);
  assert.equal(version, target.version);
  assert.equal(blob, target.authorityBlob);
  assert(managerDirectory && isAbsolute(managerDirectory));
  let pnpmPath = managerDirectory;
  const temp = process.env.RUNNER_TEMP;
  assert(temp && isAbsolute(temp));
  const stamp = process.env.GITHUB_RUN_ID + "-" + process.env.GITHUB_RUN_ATTEMPT;
  const out = join(temp, "vite-startup-evidence-" + stamp),
    projects = join(temp, "vite-startup-projects-" + stamp);
  mkdirSync(out);
  mkdirSync(projects);
  let sequence = 0;
  const save = (name: string, value: unknown) =>
    writeFileSync(join(out, name + ".json"), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
  const observed: typeof fetch = async (input, options) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url,
      response = await fetch(input, options),
      cap = /\/(?:zip|[^/]+\.tgz)$/u.test(new URL(url).pathname)
        ? 128 * 1024 * 1024
        : 4 * 1024 * 1024,
      raw = response.body ? await boundedBytes(response, cap) : Buffer.alloc(0);
    const parsed = new URL(url),
      isPublic = parsed.hostname === "registry.npmjs.org" || parsed.hostname === "api.github.com";
    save("http-" + ++sequence, {
      url: parsed.origin + parsed.pathname,
      method: options?.method ?? "GET",
      status: response.status,
      headers: {
        date: response.headers.get("date"),
        length: response.headers.get("content-length"),
        type: response.headers.get("content-type"),
      },
      bytes: raw.length,
      sha256: hash(raw),
      rawBase64:
        isPublic && !parsed.pathname.endsWith("/zip") && !parsed.pathname.endsWith(".tgz")
          ? raw.toString("base64")
          : null,
    });
    const rebuilt = new Response(response.body === null ? null : new Uint8Array(raw), {
      status: response.status,
      statusText: response.statusText,
      headers: response.headers,
    });
    for (const key of ["url", "redirected", "type"] as const)
      Object.defineProperty(rebuilt, key, { value: response[key] });
    return rebuilt;
  };
  const github = async (path: string) => {
    const response = await observed("https://api.github.com/repos/ubugeeei-prod/vize/" + path, {
      redirect: "error",
      signal: AbortSignal.timeout(30_000),
      headers: {
        Accept: "application/vnd.github+json",
        ...(process.env.GITHUB_TOKEN
          ? { Authorization: "Bearer " + process.env.GITHUB_TOKEN }
          : {}),
      },
    });
    assert.equal(response.status, 200);
    return json(Buffer.from(await response.arrayBuffer()));
  };
  const execute = (name: string, cwd: string, argv: string[]): Result =>
    collect(pnpmPath, argv, cwd, process.env, (row, stdout, stderr) => {
      writeFileSync(join(out, name + ".stdout.raw"), stdout, { flag: "wx" });
      writeFileSync(join(out, name + ".stderr.raw"), stderr, { flag: "wx" });
      save(name + ".process", row);
    });
  try {
    save("controller", {
      head: process.env.GITHUB_SHA,
      ref: process.env.GITHUB_REF,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      profile: target,
      sourcePredicatesOnly: true,
    });
    childEnvironment(process.env);
    const blobApi = await github("git/blobs/" + blob),
      capsuleRaw = Buffer.from(String(blobApi.content).replace(/\s/g, ""), "base64");
    assert.equal(blobApi.encoding, "base64");
    assert.equal(
      createHash("sha1")
        .update("blob " + capsuleRaw.length + "\0")
        .update(capsuleRaw)
        .digest("hex"),
      blob,
    );
    writeFileSync(join(out, "capsule.raw.json"), capsuleRaw, { flag: "wx" });
    const { sri, provider } = authenticateCapsule(json(capsuleRaw));
    const owned = await readReleaseOwnership(
      String(target.version),
      String(target.candidate),
      String(target.runId),
      github,
      observed,
    );
    save("release-ownership", owned);
    assert.equal(owned.producerSource, target.producerSource);
    assert.equal(owned.workflowHead, target.producerSource);
    assert.equal(owned.runAttempt, 1);
    const fix = await github("compare/" + target.requiredFix + "..." + owned.producerSource);
    assert(["ahead", "identical"].includes(String(fix.status)));
    assert.equal(object(fix.base_commit).sha, target.requiredFix);
    const authenticatePackage = packageAuthenticator(sri, observed, save);
    const route = managerRoute(managerDirectory);
    pnpmPath = route.executable;
    const managerProject = dirname(dirname(route.directory));
    await authenticatePackage(
      managerProject,
      "pnpm",
      String(object(contract.adaptation).pnpm),
      route.directory,
    );
    const exe = "@pnpm/exe.linux-x64";
    assert.equal(
      object(route.manifest.optionalDependencies)[exe],
      object(contract.adaptation).pnpm,
    );
    await authenticatePackage(
      managerProject,
      exe,
      String(object(contract.adaptation).pnpm),
      packageRoot(managerProject, exe, join(route.directory, "package.json")),
    );
    save("manager-route", {
      directory: route.directory,
      executable: route.executable,
      manifest: route.manifest,
      selectedPackage: exe,
    });
    const manager = execute("manager-version", projects, ["--version"]);
    assert.equal(manager.status, 0);
    assert.equal(manager.stderr, "");
    assert.equal(manager.stdout, object(contract.adaptation).pnpm + "\n");
    const managerImage = image(realpathSync(pnpmPath));
    save("host", {
      platform: process.platform,
      arch: process.arch,
      node: image(process.execPath),
      nodeVersion: process.version,
      pnpm: managerImage,
      writerIdentity: "UNKNOWN",
      originalHostReproduction: false,
    });
    const observations: Array<{ issue: number; state: string; result: Result; cwd: string }> = [];
    for (const issue of [11375, 11376]) {
      const project = join(projects, String(issue));
      mkdirSync(project);
      let current = originalFiles(issue);
      const writeInputs = (files: File[]) => {
        for (const f of files) {
          const path = safePath(project, f.path);
          mkdirSync(dirname(path), { recursive: true });
          writeFileSync(path, f.source);
        }
        current = files;
        verifyInputs(project, files);
      };
      writeInputs(current);
      save(issue + "-original-inputs", current);
      const installed = execute(issue + "-original-install", project, ["install"]);
      assert.equal(installed.status, 0);
      assert.equal(installed.error, null);
      assert.equal(installed.signal, null);
      verifyInputs(project, current);
      const pinned: Array<{ name: string; version: string; root: string }> = [];
      const deps = object(
        json(Buffer.from(current.find((f) => f.path === "package.json")!.source)).devDependencies,
      );
      for (const [alias, spec] of Object.entries(deps)) {
        const name = alias === "vite" && issue === 11376 ? "@voidzero-dev/vite-plus-core" : alias,
          version = alias === "vite" && issue === 11376 ? "1.4.0" : String(spec);
        const root = realpathSync(join(project, "node_modules", alias));
        await authenticatePackage(project, name, version, root);
        pinned.push({ name, version, root });
      }
      const pluginRoot = packageRoot(project, "@vizejs/vite-plugin"),
        nativeRoot = packageRoot(project, "@vizejs/native", join(pluginRoot, "package.json"));
      const platformRoot = packageRoot(
        project,
        "@vizejs/native-linux-x64-gnu",
        join(nativeRoot, "package.json"),
      );
      for (const [name, root] of [
        ["@vizejs/native", nativeRoot],
        ["@vizejs/native-linux-x64-gnu", platformRoot],
      ]) {
        await authenticatePackage(project, name!, String(target.version), root!);
        pinned.push({ name: name!, version: String(target.version), root: root! });
      }
      const nodeFile = String(provider.nodeFile),
        expectedNative = (provider.files as ObjectValue[]).find((f) => f.path === nodeFile);
      assert(expectedNative);
      const nativePath = safePath(platformRoot, nodeFile),
        native = readFileSync(nativePath);
      assert.equal(native.length, expectedNative.bytes);
      assert.equal(hash(native), expectedNative.sha256);
      assert(native.includes(Buffer.from(String(provider.nativeBuildToken))));
      const binAlias = issue === 11375 ? "vite" : "vite-plus",
        binName = issue === 11375 ? "vite" : "vp";
      const binRoot = realpathSync(join(project, "node_modules", binAlias)),
        binManifest = json(readFileSync(join(binRoot, "package.json")));
      const declared =
        typeof binManifest.bin === "string" ? binManifest.bin : object(binManifest.bin)[binName];
      assert.equal(typeof declared, "string");
      const declaredBin = safePath(binRoot, String(declared));
      const shim = join(project, "node_modules/.bin", binName);
      assert(existsSync(shim));
      assert(!existsSync(join(project, "node_modules/.bin/node")), "local Node shadow refuses");
      const selectedBin = {
        shim: image(realpathSync(shim)),
        declared: image(realpathSync(declaredBin)),
        loadedImage: "UNKNOWN",
        writerIdentity: "UNKNOWN",
      };
      save(issue + "-installation", {
        packages: pinned,
        selectedBin,
        native: image(nativePath),
        receiptProvider: provider,
        loadedNativeImage: "UNKNOWN",
        pidCustody: false,
      });
      const custody = async (label: string) => {
        verifyInputs(project, current);
        assert(!existsSync(join(project, "node_modules/.bin/node")));
        assert.deepEqual(image(realpathSync(pnpmPath)), managerImage);
        assert.deepEqual(image(realpathSync(shim)), selectedBin.shim);
        assert.deepEqual(image(realpathSync(declaredBin)), selectedBin.declared);
        for (const p of pinned) await authenticatePackage(project, p.name, p.version, p.root);
        assert.equal(hash(readFileSync(nativePath)), expectedNative.sha256);
        save(label, {
          inputs: current.map((f) => ({ path: f.path, sha256: hash(f.source) })),
          native: image(nativePath),
          packages: pinned,
        });
      };
      const states =
        issue === 11375
          ? ["original-vize", "original-official", "import-invalid", "import-repaired"]
          : [
              "original",
              "sentinel-canonical",
              "sentinel-ignore-override",
              "sentinel-scan-override",
            ];
      for (const state of states) {
        await custody(issue + "-" + state + "-prior");
        writeInputs(stateFiles(issue, state));
        save(issue + "-" + state + "-inputs", current);
        await custody(issue + "-" + state + "-pre");
        const result = execute(issue + "-" + state, project, argumentsFor(issue, state));
        observations.push({ issue, state, result, cwd: project });
        await custody(issue + "-" + state + "-post");
      }
      writeInputs(originalFiles(issue));
      await custody(issue + "-restored-final");
    }
    save("complete-observations", observations);
    assert.equal(observations.length, 8);
    const assessment = observations.map((row) => {
      try {
        verdict(row.issue, row.state, row.result, row.cwd);
        return { issue: row.issue, state: row.state, sourcePredicateMatches: true };
      } catch {
        return { issue: row.issue, state: row.state, sourcePredicateMatches: false };
      }
    });
    save("assessment", {
      assessment,
      allSourcePredicatesMatch: assessment.every((v) => v.sourcePredicateMatches),
      publicAcceptance: false,
      writerIdentity: "UNKNOWN",
      excluded: contract.excluded,
    });
    assert(
      assessment.every((v) => v.sourcePredicateMatches),
      "one or more source-owned published predicates failed",
    );
  } catch (error) {
    save("failure", {
      name: error instanceof Error ? error.name : "UNKNOWN",
      message: "bounded collector refusal; inspect preceding retained evidence",
      publicAcceptance: false,
    });
    throw new Error("published Vite startup collector refused; retained evidence precedes verdict");
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  await run();
