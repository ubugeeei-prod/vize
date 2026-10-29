import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { test } from "node:test";
import {
  nativeIpcEvidence,
  nativeIpcGrew,
  ownedProcessSnapshot,
  signalBoundProcesses,
} from "./support/lsp/diagnostic-capacity-pipe.ts";
import { waitUntil } from "./support/lsp/diagnostic-capacity.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { NativeLife } from "./support/lsp/responsive-lint-native-gate.ts";

test(
  "kernel socket calibration observes exactly queued bytes on a stopped owned receiver",
  {
    skip: process.platform !== "linux",
    timeout: 15_000,
  },
  async () => {
    const directory = fs.mkdtempSync("/tmp/tsgo-api-capacity-calibration-");
    const socket = path.join(directory, "api");
    const program =
      "require('node:net').createServer(s=>{s.pause();process.stdout.write('accepted\\n')}).listen(process.argv[1],()=>process.stdout.write('ready\\n'))";
    const child = spawn(process.execPath, ["-e", program, socket], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    let output = "",
      closed = false,
      life: NativeLife | undefined,
      client: net.Socket | undefined;
    let spawnError: Error | undefined;
    child.stdout.setEncoding("utf8").on("data", (chunk: string) => (output += chunk));
    child.once("error", (error) => (spawnError = error));
    child.once("close", () => (closed = true));
    const report: Record<string, unknown> = {
      scope: "synthetic kernel instrument calibration; not a Vize result",
      payloadBytes: 47,
    };
    try {
      assert.ok(
        await waitUntil(
          () => output.includes("ready\n") || spawnError !== undefined || closed,
          3000,
        ),
      );
      assert.equal(spawnError, undefined);
      assert.ok(output.includes("ready\n"));
      client = net.createConnection(socket);
      client.on("error", (error) => (spawnError = error));
      assert.ok(
        await waitUntil(() => output.includes("accepted\n") || spawnError !== undefined, 3000),
      );
      assert.equal(spawnError, undefined);
      life = ownedProcessSnapshot(process.pid).find((row) => row.pid === child.pid);
      assert.ok(life);
      process.kill(life.pid, "SIGSTOP");
      assert.ok(
        await waitUntil(
          () =>
            ownedProcessSnapshot(process.pid).some(
              (row) =>
                row.pid === life?.pid && row.birthTicks === life.birthTicks && row.state === "T",
            ),
          2000,
        ),
      );
      const before = nativeIpcEvidence([life]);
      report.before = before;
      assert.ok(before.channels.length > 0, "ss must join the actual owned receiver FD and inode");
      client.write("x".repeat(47));
      let after = before;
      assert.ok(
        await waitUntil(() => {
          after = nativeIpcEvidence([life!]);
          return nativeIpcGrew(before, after);
        }, 2000),
      );
      report.after = after;
      assert.ok(after.processes.every((row) => row.state === "T"));
      const grew = after.channels.find((row) =>
        before.channels.some(
          (old) =>
            old.fd === row.fd &&
            old.inode === row.inode &&
            row.queuedBytes - old.queuedBytes === 47,
        ),
      );
      assert.ok(grew, "the actual kernel must report the exact 47-byte delta");
      assert.equal(grew.localPath, socket);
      report.qualified = true;
    } catch (error) {
      report.error = String(error);
      throw error;
    } finally {
      client?.destroy();
      try {
        if (life) {
          signalBoundProcesses([life], "SIGCONT");
          signalBoundProcesses([life], "SIGTERM");
          if (!(await waitUntil(() => closed, 2000))) signalBoundProcesses([life], "SIGKILL");
        } else child.kill("SIGKILL");
      } catch (error) {
        report.cleanupError = String(error);
        child.kill("SIGCONT");
        child.kill("SIGKILL");
      }
      const terminated = await waitUntil(() => closed, 2000);
      report.cleanup = { terminated };
      report.qualified =
        report.qualified === true && terminated && report.cleanupError === undefined;
      fs.rmSync(directory, { recursive: true, force: true });
      const capture = path.join(testOutputRoot, "diagnostic-capacity-129");
      fs.mkdirSync(capture, { recursive: true });
      fs.writeFileSync(
        path.join(capture, "socket-calibration.json"),
        JSON.stringify(report, null, 2) + "\n",
      );
      assert.ok(terminated, "calibration must not leave its stopped receiver behind");
    }
  },
);
