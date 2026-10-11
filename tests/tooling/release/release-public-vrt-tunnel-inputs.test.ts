import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import {
  QuickTunnelAnnouncement,
  assertQuickTunnelBinary,
  assertQuickTunnelConfig,
  quickTunnelArguments,
  quickTunnelEnvironment,
  quickTunnelHostname,
  quickTunnelPin,
  writeQuickTunnelConfig,
} from "../../../tools/support/release/public_acceptance/vrt_tunnel.ts";

async function owned(t: TestContext) {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-quick-input-control-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  return root;
}

// Input/file controls only. No cloudflared executable, account, browser or tunnel is used.
test("explicit owned empty config and minimal argv cannot silently consume personal settings", async (t) => {
  const root = await owned(t);
  const task = path.join(root, "task");
  const config = await writeQuickTunnelConfig(task);
  await assertQuickTunnelConfig(task, config);
  assert.equal(await readFile(config, "utf8"), "{}\n");
  assert.deepEqual(quickTunnelArguments(config, "http://127.0.0.1:43719"), [
    "tunnel",
    "--config",
    config,
    "--no-autoupdate",
    "--metrics",
    "127.0.0.1:0",
    "--url",
    "http://127.0.0.1:43719",
  ]);
  await assert.rejects(writeQuickTunnelConfig(task), { code: "EEXIST" });
  assert.equal(await readFile(config, "utf8"), "{}\n");
  await assert.rejects(writeQuickTunnelConfig("relative-control"), /Absolute owned/);
});

test("missing, changed, permissive, foreign and symlink config refuses", async (t) => {
  for (const kind of ["missing", "changed", "permissions", "foreign", "symlink"]) {
    const root = await owned(t);
    const task = path.join(root, "task");
    const config = await writeQuickTunnelConfig(task);
    let selected = config;
    if (kind === "missing") await rm(config);
    if (kind === "changed") await writeFile(config, "tunnel: different\n");
    if (kind === "permissions") {
      const { chmod } = await import("node:fs/promises");
      await chmod(config, 0o644);
    }
    if (kind === "foreign") {
      selected = path.join(root, "foreign.yml");
      await writeFile(selected, "{}\n", { mode: 0o600 });
    }
    if (kind === "symlink") {
      await rm(config);
      const target = path.join(root, "target.yml");
      await writeFile(target, "{}\n", { mode: 0o600 });
      await symlink(target, config);
    }
    await assert.rejects(assertQuickTunnelConfig(task, selected), kind);
  }
});

test("child environment preserves input HOME and drops all unrelated credential/trust/debug values", () => {
  const original = {
    HOME: "/unchanged-control-home",
    PATH: "/control/bin",
    TMPDIR: "/control/tmp",
    LANG: "C.UTF-8",
    TUNNEL_TOKEN: "control-secret",
    TUNNEL_NAME: "control-name",
    TUNNEL_ORIGIN_CERT: "control.pem",
    TUNNEL_LOGLEVEL: "debug",
    TUNNEL_LOGFILE: "control.log",
    TUNNEL_ALLOWED_MAIL: "control@example.invalid",
    TUNNEL_QUICK_SERVICE: "https://control.invalid",
    HTTP_PROXY: "http://control.invalid",
    SSL_CERT_FILE: "control.pem",
    NODE_OPTIONS: "control-override",
    GH_TOKEN: "control-secret",
  };
  const before = { ...original };
  const env = quickTunnelEnvironment(original);
  assert.deepEqual(env, {
    HOME: original.HOME,
    PATH: original.PATH,
    TMPDIR: original.TMPDIR,
    LANG: original.LANG,
  });
  assert.deepEqual(original, before);
  assert.throws(() => quickTunnelEnvironment({}), /Original HOME/);
});

test("only the exact static HTTP loopback origin and anonymous HTTPS hostname are accepted", () => {
  const config = "/owned-control/config.yml";
  for (const origin of [
    "http://localhost:43719",
    "http://127.0.0.1",
    "http://127.0.0.1:43719/gallery/",
    "http://127.0.0.1:43719?token=control",
    "http://user:control@127.0.0.1:43719",
    "https://127.0.0.1:43719",
    " http://127.0.0.1:43719",
    "http://127.0.0.1:99999",
    "invalid",
  ])
    assert.throws(() => quickTunnelArguments(config, origin));
  assert.equal(
    quickTunnelHostname("https://a-b.trycloudflare.com"),
    "https://a-b.trycloudflare.com",
  );
  for (const value of [
    "http://a.trycloudflare.com",
    "https://api.trycloudflare.com",
    "https://trycloudflare.com",
    "https://a.trycloudflare.com.evil.invalid",
    "https://a.trycloudflare.com/path",
    "https://a.trycloudflare.com?token=control",
    "https://user:control@a.trycloudflare.com",
    "https://a.trycloudflare.com:8443",
    "https://-a.trycloudflare.com",
    "invalid",
  ])
    assert.throws(() => quickTunnelHostname(value));
});

test("tagged announcement rows establish one hostname and refuse reprovisioning or changes", () => {
  const announced = new QuickTunnelAnnouncement();
  assert.equal(announced.line("unrelated https://unowned.trycloudflare.com"), undefined);
  assert.equal(
    announced.line(
      "INF | Your quick Tunnel has been created! Visit it at (it may take some time to be reachable): |",
    ),
    undefined,
  );
  assert.equal(
    announced.line("INF | https://a-b.trycloudflare.com   |"),
    "https://a-b.trycloudflare.com",
  );
  assert.equal(
    announced.line("INF | https://a-b.trycloudflare.com   |"),
    "https://a-b.trycloudflare.com",
  );
  assert.throws(
    () => announced.line("INF | https://different.trycloudflare.com |"),
    /hostname changed/,
  );
  assert.throws(
    () => announced.line("INF | Your quick Tunnel has been created! Visit it at |"),
    /Repeated/,
  );
  assert.throws(() => new QuickTunnelAnnouncement().line("x".repeat(16_385)), /too large/);
});

test("binary length/digest and normal-file ownership refuse before any execution", async (t) => {
  const root = await owned(t);
  const assets = path.join(root, "assets");
  await mkdir(assets);
  const binary = path.join(assets, "cloudflared-linux-amd64");
  await writeFile(binary, "control bytes");
  await assert.rejects(assertQuickTunnelBinary(assets, binary), /length mismatch/);
  // Non-executable control bytes exercise the real SHA algorithm, never a replacement binary.
  await writeFile(binary, Buffer.alloc(quickTunnelPin.bytes));
  await assert.rejects(assertQuickTunnelBinary(assets, binary), /digest mismatch/);
  const link = path.join(root, "cloudflared-linux-amd64");
  await symlink(binary, link);
  await assert.rejects(assertQuickTunnelBinary(root, link));
});
