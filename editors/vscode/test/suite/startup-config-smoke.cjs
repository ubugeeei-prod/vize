const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const {
  assertInitializationOptions,
  disableVizeAndWaitForShutdown,
  getFakeServer,
  initializeMessages,
  methodMessages,
  prepareConfiguredFakeServer,
  updateVizeConfiguration,
  waitForLogEntries,
  waitForReadyServer,
} = require("./extension-host-support.cjs");

exports.runStartupConfigSmoke = async function runStartupConfigSmoke() {
  const { logPath, serverPath } = getFakeServer();
  const previousGate = process.env.VIZE_TEST_SERVER_INITIALIZE_GATE;
  const directory = path.join(path.dirname(logPath), "existing-config-events", "pending-startup");
  fs.mkdirSync(directory, { recursive: true });
  const gate = path.join(directory, "initialize-release");
  const options = { typecheck: false };

  try {
    for (const disableWhilePending of [true, false]) {
      await prepareConfiguredFakeServer({ logPath, serverPath });
      fs.rmSync(gate, { force: true });
      process.env.VIZE_TEST_SERVER_INITIALIZE_GATE = gate;
      await updateVizeConfiguration("typecheck.enable", false);
      await updateVizeConfiguration("enable", true);
      let entries = await waitForLogEntries(
        logPath,
        (next) => next.some((entry) => entry.event === "initialize-gated"),
        "controlled pending startup",
      );
      assert.equal(initializeMessages(entries).length, 1);
      assert.equal(methodMessages(entries, "initialized").length, 0);
      assertInitializationOptions(entries, options);
      const pid = entries.find(
        (entry) => entry.event === "process-start" && entry.args[0] === "lsp",
      ).pid;
      const prefix = path.join(directory, disableWhilePending ? "disabled" : "completed");
      fs.copyFileSync(logPath, `${prefix}.before.jsonl`);
      try {
        if (disableWhilePending) {
          const disable = vscode.commands.executeCommand("vize.disable");
          await waitForLogEntries(
            logPath,
            () => vscode.workspace.getConfiguration("vize").get("enable") === false,
            "disable while initialize is pending",
          );
          fs.writeFileSync(gate, "release\n");
          await disable;
        } else {
          fs.writeFileSync(gate, "release\n");
          entries = await waitForReadyServer(logPath, "controlled startup completes");
          assertInitializationOptions(entries, options);
          await disableVizeAndWaitForShutdown(logPath);
        }
        entries = await waitForLogEntries(
          logPath,
          (next) => next.some((entry) => entry.event === "process-exit" && entry.pid === pid),
          "disabled startup owns and reaps its server process",
        );
        assert.equal(initializeMessages(entries).length, 1);
        assert.equal(methodMessages(entries, "shutdown").length, 1);
        assert.equal(methodMessages(entries, "exit").length, 1);
        assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
        await assert.rejects(vscode.commands.executeCommand("vize.test.getServerInfo"));
      } finally {
        fs.writeFileSync(gate, "release\n");
        fs.copyFileSync(logPath, `${prefix}.after.jsonl`);
      }
    }
  } finally {
    if (previousGate === undefined) delete process.env.VIZE_TEST_SERVER_INITIALIZE_GATE;
    else process.env.VIZE_TEST_SERVER_INITIALIZE_GATE = previousGate;
  }
};
