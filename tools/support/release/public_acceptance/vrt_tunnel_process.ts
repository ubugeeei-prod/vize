import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { StringDecoder } from "node:string_decoder";
import {
  assertQuickTunnelBinary,
  assertQuickTunnelConfig,
  quickTunnelArguments,
  quickTunnelEnvironment,
  quickTunnelPin,
  QuickTunnelAnnouncement,
} from "./vrt_tunnel.ts";

interface Exit {
  code: number | null;
  signal: NodeJS.Signals | null;
  category?: string;
}

/** Preparation only: callers must separately authenticate approved acquisition and executable help. */
export async function startStaticQuickTunnel(input: {
  binary: string;
  assetDirectory: string;
  config: string;
  configDirectory: string;
  staticOrigin: string;
}) {
  await assertQuickTunnelBinary(input.assetDirectory, input.binary);
  await assertQuickTunnelConfig(input.configDirectory, input.config);
  const args = quickTunnelArguments(input.config, input.staticOrigin);
  const environment = quickTunnelEnvironment(process.env);
  assert.equal(environment.HOME, process.env.HOME);
  const announcement = new QuickTunnelAnnouncement();
  let accept!: (value: string) => void;
  let rejectReady!: (error: Error) => void;
  const ready = new Promise<string>((resolve, reject) => {
    accept = resolve;
    rejectReady = reject;
  });
  // No credential argument/environment, shell, saved stream, retry, update or browser launch.
  const child = spawn(input.binary, args, {
    cwd: input.configDirectory,
    env: environment,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let exit: Exit | undefined;
  let failure: Error | undefined;
  let hostname: string | undefined;
  let finish!: (value: Exit) => void;
  const closed = new Promise<Exit>((resolve) => {
    finish = resolve;
  });
  const buffers = ["", ""];
  function refuse(category: string) {
    failure ??= new Error(category);
    rejectReady(failure);
    if (!exit) child.kill("SIGTERM");
  }
  child.once("error", () => refuse("Pinned Quick Tunnel child startup failed"));
  child.once("close", (code, signal) => {
    exit = { code, signal, category: failure?.message };
    buffers.fill("");
    finish(exit);
    if (!hostname) rejectReady(new Error("Quick Tunnel closed before anonymous announcement"));
  });
  for (const [index, stream] of [child.stdout, child.stderr].entries()) {
    const decoder = new StringDecoder("utf8");
    function consume(value: string) {
      if (failure || exit) return;
      buffers[index] += value;
      try {
        let split: number;
        while ((split = buffers[index].indexOf("\n")) >= 0) {
          const line = buffers[index].slice(0, split);
          buffers[index] = buffers[index].slice(split + 1);
          const url = announcement.line(line);
          if (url) {
            hostname = url;
            accept(url);
          }
        }
        assert.ok(Buffer.byteLength(buffers[index]) <= 16_384);
      } catch {
        buffers[index] = "";
        refuse("Quick Tunnel announcement or bounded stream contract failed");
      }
    }
    stream.on("data", (bytes: Buffer) => consume(decoder.write(bytes)));
    stream.once("end", () => {
      consume(decoder.end());
      if (buffers[index]) consume("\n");
      buffers[index] = "";
    });
    stream.once("error", () => refuse("Quick Tunnel pipe drainage failed"));
  }
  async function waitForClose(milliseconds: number): Promise<Exit | undefined> {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      return await Promise.race([
        closed,
        new Promise<undefined>((resolve) => {
          timer = setTimeout(() => resolve(undefined), milliseconds);
        }),
      ]);
    } finally {
      if (timer) clearTimeout(timer);
    }
  }
  async function close(): Promise<Exit> {
    if (exit) return exit;
    child.kill("SIGTERM");
    const graceful = await waitForClose(5_000);
    if (graceful) return graceful;
    child.kill("SIGKILL");
    const forced = await waitForClose(5_000);
    assert.ok(forced, "Owned Quick Tunnel child and pipes did not close");
    return forced;
  }
  const timeout = setTimeout(() => refuse("Quick Tunnel announcement deadline expired"), 30_000);
  try {
    const origin = await ready;
    assert.ok(child.pid && !failure && !exit, "Quick Tunnel announcement lost process ownership");
    return {
      origin,
      pid: child.pid,
      version: quickTunnelPin.version,
      /** An announcement is not TLS, reachability, Public address-space or installed acceptance. */
      assertRunning() {
        assert.ok(!failure && !exit, "Owned Quick Tunnel stopped or refused");
      },
      close,
    };
  } catch (primary) {
    try {
      await close();
    } catch {
      throw new AggregateError(
        [primary, new Error("Quick Tunnel cleanup failed")],
        "Pinned static tunnel acquisition and cleanup refused",
      );
    }
    throw primary;
  } finally {
    clearTimeout(timeout);
  }
}
