import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  collectBinConfiguration,
  globalBinConfigPath,
  recheckBinConfiguration,
  validateBinConfiguration,
  safeAgeWorkspace,
} from "../../../tools/support/release/human_reports_11252_11253_public/bin-configuration.ts";
import { configurationFixture } from "./human-reports-11252-11253-bin.fixture.ts";
import { sha256 } from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import { cleanEnvironment } from "../../../tools/support/release/human_reports_11252_11253_public/capture.ts";

const manifest = Buffer.from(
  '{"name":"synthetic-original-consumer","devDependencies":{"vize":"0.0.0"}}\n',
);
function setup() {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "human-config-inputs-")));
  const root = join(base, "consumer-original"),
    xdg = join(base, "xdg");
  mkdirSync(root);
  writeFileSync(join(root, "package.json"), manifest);
  const user = join(base, "original-user.npmrc"),
    global = join(base, "original-global.npmrc");
  writeFileSync(user, "");
  writeFileSync(global, "");
  const env: NodeJS.ProcessEnv = {
    HOME: homedir(),
    XDG_CONFIG_HOME: xdg,
    npm_config_registry: "https://registry.npmjs.org",
    npm_config_userconfig: user,
    npm_config_globalconfig: global,
  };
  const events: Array<{ kind: string; value: unknown }> = [];
  const sink = (kind: string, value: unknown) => events.push({ kind, value });
  return { base, root, xdg, user, global, env, events, sink };
}
test("source config-dir branches are exact and unknown empty/relative selectors refuse", () => {
  assert.equal(
    globalBinConfigPath("darwin", "/home/u", { XDG_CONFIG_HOME: null }),
    "/home/u/Library/Preferences/pnpm/config.yaml",
  );
  assert.equal(
    globalBinConfigPath("linux", "/home/u", { XDG_CONFIG_HOME: null }),
    "/home/u/.config/pnpm/config.yaml",
  );
  for (const os of ["darwin", "linux"])
    assert.equal(
      globalBinConfigPath(os, "/home/u", { XDG_CONFIG_HOME: "/explicit" }),
      "/explicit/pnpm/config.yaml",
    );
  for (const path of ["", "relative", "/x/../y"])
    assert.throws(() => globalBinConfigPath("linux", "/home/u", { XDG_CONFIG_HOME: path }));
  assert.throws(() => globalBinConfigPath("win32", "/home/u", {}));
});
test("preinstall collection preserves original files and uses the actual absent workspace walk without creating settings", () => {
  const s = setup();
  try {
    const input = collectBinConfiguration(s.root, s.env, manifest, s.sink);
    assert(input.workspace.every((row) => row.state === "ABSENT"));
    assert.deepEqual(readFileSync(join(s.root, "package.json")), manifest);
    assert.equal(input.global.path, join(s.xdg, "pnpm/config.yaml"));
    assert.equal(input.global.state, "ABSENT");
    assert.equal(input.writerRuntimeAttribution, false);
    assert(s.events.some((row) => row.kind === "bin-configuration-before-install"));
    writeFileSync(
      join(s.root, "pnpm-workspace.yaml"),
      "minimumReleaseAgeExclude:\n  - 'vize@0.0.0'\n",
    );
    recheckBinConfiguration(input, manifest, s.sink);
    writeFileSync(join(s.root, "pnpm-workspace.yaml"), "nodeLinker: hoisted\n");
    assert.throws(() => recheckBinConfiguration(input, manifest, s.sink));
    assert(s.events.some((row) => row.kind === "bin-configuration-after-install-or-command"));
  } finally {
    rmSync(s.base, { recursive: true, force: true });
  }
});
test("every PNPM namespace override is retained and refuses, including conflicting case and empty masking", () => {
  for (const additions of [
    { PNPM_CONFIG_NODE_LINKER: "hoisted" },
    { pnpm_config_modules_dir: "elsewhere" },
    { PNPM_CONFIG_NODE_LINKER: "", pnpm_config_node_linker: "hoisted" },
    { pnpm_config_userconfig: "", PNPM_CONFIG_USERCONFIG: "/foreign" },
    { PNPM_CONFIG_WORKSPACE_DIR: "", pnpm_config_workspace_dir: "/foreign" },
    { NPM_CONFIG_WORKSPACE_DIR: "/foreign" },
  ]) {
    const s = setup();
    try {
      Object.assign(s.env, additions);
      assert.throws(() => collectBinConfiguration(s.root, s.env, manifest, s.sink));
      const event = s.events.find(
        (row) => row.kind === "bin-configuration-environment-before-assertion",
      );
      assert(event);
      for (const name of Object.keys(additions)) assert(JSON.stringify(event.value).includes(name));
    } finally {
      rmSync(s.base, { recursive: true, force: true });
    }
  }
});
test("unowned global and ancestor config is hashed before refusal without exporting credential bodies", () => {
  for (const kind of ["global", "ancestor"] as const) {
    const s = setup();
    try {
      const path =
        kind === "global" ? join(s.xdg, "pnpm/config.yaml") : join(s.base, "pnpm-workspace.yaml");
      mkdirSync(join(path, ".."), { recursive: true });
      const secret = "synthetic-private-auth-value";
      writeFileSync(path, "token: " + secret + "\n");
      assert.throws(() => collectBinConfiguration(s.root, s.env, manifest, s.sink));
      const serialized = JSON.stringify(s.events);
      assert(serialized.includes(sha256("token: " + secret + "\n")));
      assert(!serialized.includes(secret));
      assert(serialized.includes(path));
    } finally {
      rmSync(s.base, { recursive: true, force: true });
    }
  }
  const s = setup();
  try {
    s.env.PNPM_CONFIG_NPM_TOKEN = "synthetic-secret";
    assert.throws(() => collectBinConfiguration(s.root, s.env, manifest, s.sink));
    assert(!JSON.stringify(s.events).includes("synthetic-secret"));
  } finally {
    rmSync(s.base, { recursive: true, force: true });
  }
});
test("missing/duplicate source rows, wrong resolved path, extra flags and package config cannot derive options", () => {
  for (const mutate of [
    (input: ReturnType<typeof configurationFixture>) => {
      input.workspace.pop();
    },
    (input: ReturnType<typeof configurationFixture>) => {
      input.workspace.push(input.workspace[0]!);
    },
    (input: ReturnType<typeof configurationFixture>) => {
      input.global.path = "/wrong/config.yaml";
    },
    (input: ReturnType<typeof configurationFixture>) => {
      input.argv.push("--config.node-linker=hoisted");
    },
    (input: ReturnType<typeof configurationFixture>) => {
      input.configDirPreseeded = true as false;
    },
  ]) {
    const input = configurationFixture("/inert/consumer-original", "linux", manifest);
    mutate(input);
    assert.throws(() => validateBinConfiguration(input, manifest));
  }
  const changed = Buffer.from('{"pnpm":{"nodeLinker":"hoisted"}}\n');
  assert.throws(() =>
    validateBinConfiguration(
      configurationFixture("/inert/consumer-original", "linux", changed),
      changed,
    ),
  );
});
test("postinstall change to global or controlled npmrc is retained before refusal", () => {
  for (const kind of ["global", "user"] as const) {
    const s = setup();
    try {
      const input = collectBinConfiguration(s.root, s.env, manifest, s.sink);
      const path = kind === "global" ? join(s.xdg, "pnpm/config.yaml") : s.user;
      mkdirSync(join(path, ".."), { recursive: true });
      writeFileSync(path, "changed\n");
      assert.throws(() => recheckBinConfiguration(input, manifest, s.sink));
      assert(s.events.some((row) => row.kind === "bin-configuration-after-install-or-command"));
    } finally {
      rmSync(s.base, { recursive: true, force: true });
    }
  }
});
test("unexpected controlled npmrc/package/workspace bytes are hashed and never emitted before refusal", () => {
  const secret = "synthetic-unexpected-token-body";
  for (const when of ["before", "after"] as const)
    for (const kind of ["user", "global", "package", "workspace"] as const) {
      const s = setup();
      try {
        const input =
          when === "after" ? collectBinConfiguration(s.root, s.env, manifest, s.sink) : null;
        const path =
          kind === "user"
            ? s.user
            : kind === "global"
              ? s.global
              : join(s.root, kind === "package" ? "package.json" : "pnpm-workspace.yaml");
        const body =
          kind === "workspace"
            ? `minimumReleaseAgeExclude:\n  - '${secret}'\n`
            : `token=${secret}\n`;
        writeFileSync(path, body);
        assert.throws(() =>
          input
            ? recheckBinConfiguration(input, manifest, s.sink)
            : collectBinConfiguration(s.root, s.env, manifest, s.sink),
        );
        const raw = JSON.stringify(s.events);
        assert(raw.includes(sha256(body)));
        assert(!raw.includes(secret));
        assert(!raw.includes(Buffer.from(body).toString("base64")));
      } finally {
        rmSync(s.base, { recursive: true, force: true });
      }
    }
});
test("age workspace allows only unique release-bound package values, including pnpm's unquoted vize scalar", () => {
  assert(
    safeAgeWorkspace(
      Buffer.from("minimumReleaseAgeExclude:\n  - '@vizejs/native@0.0.0'\n  - vize@0.0.0\n"),
      manifest,
    ),
  );
  for (const body of [
    "minimumReleaseAgeExclude:\n  - arbitrary-secret\n",
    "minimumReleaseAgeExclude:\n  - 'vize@1.0.0'\n",
    "minimumReleaseAgeExclude:\n  - vize@0.0.0\n  - vize@0.0.0\n",
    "minimumReleaseAgeExclude:\n  - vize@0.0.0\nnodeLinker: hoisted\n",
  ])
    assert(!safeAgeWorkspace(Buffer.from(body), manifest));
});
test("real cleanup caller witnesses and rejects ambient NPM/PNPM overrides before they can disappear", () => {
  const secret = "synthetic-ambient-private-value";
  for (const name of [
    "npm_config_node_linker",
    "NPM_CONFIG_NODE_LINKER",
    "pnpm_config_node_linker",
    "PNPM_CONFIG_NODE_LINKER",
  ])
    for (const value of [secret, ""]) {
      const ambient = { PATH: "/synthetic/bin", [name]: value },
        before = { ...ambient };
      const events: Array<{ kind: string; value: unknown }> = [];
      assert.throws(() => cleanEnvironment((kind, value) => events.push({ kind, value }), ambient));
      assert.deepEqual(ambient, before, "refusal must not delete or rewrite inherited settings");
      assert.equal(events[0]?.kind, "ambient-package-manager-configuration-before-cleanup");
      const raw = JSON.stringify(events);
      assert(raw.includes(name));
      assert(raw.includes(sha256(value)));
      assert(raw.includes(`"empty":${value === ""}`));
      assert(!raw.includes(secret));
      assert(!raw.includes(Buffer.from(secret).toString("base64")));
    }
});
test("accepted ambient caller keeps safe settings unchanged and only existing credential removal applies", () => {
  const ambient = {
    PATH: "/synthetic/bin",
    HOME: "/synthetic/home",
    PNPM_HOME: "/synthetic/pnpm",
    CI: "true",
    GITHUB_TOKEN: "synthetic-auth",
  };
  const events: Array<{ kind: string; value: unknown }> = [];
  const cleaned = cleanEnvironment((kind, value) => events.push({ kind, value }), ambient);
  assert.deepEqual(cleaned, {
    PATH: ambient.PATH,
    HOME: ambient.HOME,
    PNPM_HOME: ambient.PNPM_HOME,
    CI: "true",
  });
  assert.equal(ambient.GITHUB_TOKEN, "synthetic-auth");
  assert.deepEqual(events, [
    { kind: "ambient-package-manager-configuration-before-cleanup", value: { rows: [] } },
  ]);
});
