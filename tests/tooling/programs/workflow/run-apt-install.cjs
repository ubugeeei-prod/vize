const assert = require("node:assert/strict");
const fs = require("node:fs");
const { spawnSync } = require("node:child_process");
const dir = process.env.PROBE_DIR;
const names = fs.readdirSync(dir + "/config");
const ownCache = names.filter(
  (name) => name.startsWith("99-vize-nuxt-archives-") && !name.endsWith("original"),
);
assert.equal(ownCache.length, 1);
const config = fs.readFileSync(dir + "/config/" + ownCache[0], "utf8");
const archive = /Dir::Cache::archives "([^"]+)";/.exec(config)[1];
const hook = /APT::Update::Post-Invoke \{ "'([^']+)'"; \};/.exec(config)[1];
assert.equal(hook, archive + "/report-inventory");
const record = (value) => fs.appendFileSync(dir + "/events", value + "\n");
const update = () => {
  // The existing Docker-clean fixed-path hook remains active before our observer.
  fs.rmSync(dir + "/cache/archives/default.deb", { force: true });
  assert.equal(spawnSync(hook, [], { stdio: "inherit" }).status, 0);
};
const args = process.argv.slice(2);
const ownDownload = names.filter(
  (name) => name.startsWith("98-vize-nuxt-download-only-") && !name.endsWith("original"),
);
if (JSON.stringify(args) === JSON.stringify(["exec", "playwright", "install-deps", "chromium"])) {
  assert.equal(ownDownload.length, 1);
  assert.equal(
    fs.readFileSync(dir + "/config/" + ownDownload[0], "utf8"),
    'APT::Get::Download-Only "true";\n',
  );
  assert.equal(fs.readFileSync(dir + "/ready", "utf8"), "ready\n");
  record("download-start");
  update();
  fs.writeFileSync(archive + "/package.deb", "verified package archive\n");
  record("download-done");
  if (process.env.CANCEL === "1") {
    // Close the FIFO open/read before testing SIGTERM. Keep the CLI alive for the signal.
    fs.writeFileSync(dir + "/gate", "cancel-wait\n");
    process.kill(Number(fs.readFileSync(dir + "/parent-pid", "utf8")), "SIGTERM");
    process.exit(0);
  }
  fs.writeFileSync(dir + "/gate", "done\n");
  process.exit(Number(process.env.DOWNLOAD_STATUS));
}
assert.deepEqual(args, ["exec", "playwright", "install", "--with-deps", "chromium"]);
assert.equal(ownDownload.length, 0);
assert.ok(fs.readFileSync(dir + "/events", "utf8").includes("cli-done\n"));
record("install-start");
update();
assert.equal(fs.readFileSync(archive + "/package.deb", "utf8"), "verified package archive\n");
record("archive-reused");
process.exit(Number(process.env.INSTALL_STATUS));
