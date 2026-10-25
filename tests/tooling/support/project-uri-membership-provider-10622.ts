import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { after } from "node:test";
import { root, testOutputRoot } from "./lsp/paths.ts";

const fixture = path.join(root, "tests/_fixtures/differential/lsp/project-uri-membership-10622");
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const authorityBytes = fs.readFileSync(path.join(fixture, "authority.json"));
assert.equal(
  digest(authorityBytes),
  "d9659038353d4eb8181baa0677ae296ad6ced6d8687d25873df60434eb660395",
);
const authority = JSON.parse(authorityBytes.toString());

export type VueProvider = {
  path: string;
  name: "vue";
  version: "3.5.41";
  packageJsonSha256: string;
};
type Request = {
  requestedSource: boolean;
  dependenciesRequired: boolean;
  explicitVue: string | undefined;
};
type Installed = { path: string; modules: string };

/** A supplied path is authoritative: malformed paths and wrong versions never trigger an install. */
export function validateProjectUriVue(value: string): VueProvider {
  assert.ok(value.length > 0, "VIZE_PROJECT_URI_VUE_PACKAGE must select a Vue package");
  const selected = fs.realpathSync(value);
  assert.ok(fs.statSync(selected).isDirectory(), "the explicit Vue provider must be a directory");
  const bytes = fs.readFileSync(path.join(selected, "package.json"));
  const manifest = JSON.parse(bytes.toString());
  assert.equal(manifest.name, "vue");
  assert.equal(
    manifest.version,
    "3.5.41",
    "the whole-packet oracle requires the frozen real Vue version",
  );
  assert.equal(
    digest(bytes),
    authority.vuePackageJsonSha256,
    "the real Vue manifest must match published evidence",
  );
  return { path: selected, name: "vue", version: "3.5.41", packageJsonSha256: digest(bytes) };
}

export function resolveProjectUriVue(request: Request, install: () => Installed) {
  assert.ok(
    !request.requestedSource || request.dependenciesRequired,
    "source requests require the pinned dependencies",
  );
  if (request.explicitVue !== undefined)
    return { mode: "explicit" as const, provider: validateProjectUriVue(request.explicitVue) };
  if (!request.dependenciesRequired) return { mode: "not-requested" as const, provider: undefined };
  const installed = install();
  return {
    mode: "installed" as const,
    provider: validateProjectUriVue(installed.path),
    modules: installed.modules,
  };
}

function installPinned(directory: string): Installed {
  const installation = path.join(directory, "installation");
  fs.mkdirSync(installation);
  for (const file of ["package.json", "package-lock.json"]) {
    const key = `vue-3.5.41/${file}`;
    const bytes = fs.readFileSync(path.join(fixture, key));
    assert.equal(digest(bytes), authority.files[key], `frozen provider input ${key}`);
    fs.writeFileSync(path.join(installation, file), bytes);
  }
  const argv = ["ci", "--ignore-scripts", "--no-audit", "--no-fund"];
  const installed = spawnSync(process.platform === "win32" ? "npm.cmd" : "npm", argv, {
    cwd: installation,
    encoding: "utf8",
  });
  fs.writeFileSync(path.join(directory, "npm.stdout.log"), installed.stdout ?? "");
  fs.writeFileSync(path.join(directory, "npm.stderr.log"), installed.stderr ?? "");
  fs.writeFileSync(
    path.join(directory, "install.json"),
    JSON.stringify(
      {
        argv: ["npm", ...argv],
        cwd: installation,
        exitStatus: installed.status,
        signal: installed.signal,
        error: installed.error?.message ?? null,
        packageSha256: digest(fs.readFileSync(path.join(installation, "package.json"))),
        lockSha256: digest(fs.readFileSync(path.join(installation, "package-lock.json"))),
        sourceAcceptance: false,
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(installed.error, undefined, "the frozen provider install must execute successfully");
  assert.equal(installed.signal, null);
  assert.equal(
    installed.status,
    0,
    "the frozen provider install must succeed; no dependency fallback exists",
  );
  for (const file of ["package.json", "package-lock.json"])
    assert.equal(
      digest(fs.readFileSync(path.join(installation, file))),
      authority.files[`vue-3.5.41/${file}`],
      "npm ci must preserve the frozen provider inputs",
    );
  return {
    path: path.join(installation, "node_modules/vue"),
    modules: path.join(installation, "node_modules"),
  };
}

type Prepared = {
  vue: string | undefined;
  requestedSource: boolean;
  dependenciesRequired: boolean;
  receipt: string;
  openSession(): () => void;
};
let prepared: Prepared | undefined;

/** One explicit or isolated provider per test process, prepared before any AST correctness laws. */
export function prepareProjectUriProvider(): Prepared {
  if (prepared) return prepared;
  const request: Request = {
    requestedSource: process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD === "1",
    dependenciesRequired:
      process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD === "1" ||
      process.env.VIZE_TEST_REQUIRE_TSGO === "1",
    explicitVue: process.env.VIZE_PROJECT_URI_VUE_PACKAGE,
  };
  const parent = path.join(testOutputRoot, "project-uri-membership-10622", "providers");
  fs.mkdirSync(parent, { recursive: true });
  const directory = fs.mkdtempSync(path.join(parent, `process-${process.pid}-`));
  const receipt = path.join(directory, "provider.json");
  const state = {
    schema: "vize.lsp.project-uri.test-provider",
    version: 1,
    processId: process.pid,
    requestedSource: request.requestedSource,
    dependenciesRequired: request.dependenciesRequired,
    explicitProviderSupplied: request.explicitVue !== undefined,
    dependencyAvailable: false,
    availabilityBeforeTests: false,
    sourceAcceptance: false,
    activeSessions: 0,
    openedSessions: 0,
    closedSessions: 0,
    state: "preparing",
    provider: undefined as VueProvider | undefined,
    mode: "pending",
    error: null as string | null,
    cleanup: "not-created",
  };
  const save = () => fs.writeFileSync(receipt, JSON.stringify(state, null, 2) + "\n");
  save();
  let selected: ReturnType<typeof resolveProjectUriVue>;
  try {
    selected = resolveProjectUriVue(request, () => installPinned(directory));
    state.provider = selected.provider;
    state.dependencyAvailable = selected.provider !== undefined;
    state.availabilityBeforeTests = state.dependencyAvailable;
    state.mode = selected.mode;
    state.state = selected.provider
      ? "dependency-ready"
      : "source-not-requested-provider-unavailable";
    state.cleanup =
      selected.mode === "installed"
        ? "waiting-for-all-sessions"
        : selected.mode === "explicit"
          ? "external-provider-preserved"
          : "not-created";
    if (selected.provider) process.env.VIZE_PROJECT_URI_VUE_PACKAGE = selected.provider.path;
    save();
  } catch (error) {
    state.state = "failed";
    state.error = String(error);
    save();
    throw error;
  }
  prepared = {
    vue: selected.provider?.path,
    requestedSource: request.requestedSource,
    dependenciesRequired: request.dependenciesRequired,
    receipt,
    openSession() {
      assert.ok(
        request.requestedSource && selected.provider,
        "only source-required cases can borrow this provider",
      );
      state.activeSessions++;
      state.openedSessions++;
      save();
      let closed = false;
      return () => {
        assert.equal(closed, false, "one observed process close releases one provider lease");
        closed = true;
        state.activeSessions--;
        state.closedSessions++;
        save();
      };
    },
  };
  after(() => {
    if (selected.mode === "installed") {
      if (state.activeSessions !== 0) state.cleanup = "retained-because-a-session-did-not-close";
      else {
        assert.equal(state.openedSessions, state.closedSessions);
        assert.equal(path.dirname(selected.modules), path.join(directory, "installation"));
        assert.ok(!fs.lstatSync(selected.modules).isSymbolicLink());
        fs.rmSync(selected.modules, { recursive: true, force: true });
        state.dependencyAvailable = false;
        state.cleanup = "owned-modules-removed-after-all-observed-session-closes";
      }
    }
    state.state = "test-process-finished";
    save();
  });
  return prepared;
}
