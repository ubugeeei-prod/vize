import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import path from "node:path";
import { StringDecoder } from "node:string_decoder";
import { rejectOverrides } from "./installed.ts";
import { installedVrtBin } from "./vrt_cli.ts";
import type { VrtFixture } from "./vrt_fixtures.ts";
import { VrtSecrets } from "./vrt_secrets.ts";
import { PublishedVrtAnnouncement } from "./vrt_service_announcement.ts";
import { quickTunnelEnvironment, quickTunnelHostname } from "./vrt_tunnel.ts";

const live = new Set<number>();
export function assertPublishedVrtChildrenClosed(): void {
  assert.equal(live.size, 0, "Owned published VRT child/pipes remain; writer barrier refused");
}

/** Source preparation only: the official installation must actually include this serve protocol. */
export async function startPublishedVrtSession(
  fixture: Pick<VrtFixture, "root" | "config">,
  galleryUrl: string,
  secrets: VrtSecrets,
) {
  rejectOverrides();
  const gallery = new URL(galleryUrl);
  quickTunnelHostname(gallery.origin);
  assert.ok(
    !gallery.username &&
      !gallery.password &&
      !gallery.search &&
      !gallery.hash &&
      galleryUrl === gallery.href &&
      gallery.pathname.endsWith("/"),
    "Fixed public gallery required",
  );
  const bin = await installedVrtBin();
  const args = [
    bin,
    "serve",
    "--gallery-url",
    galleryUrl,
    "--config",
    fixture.config,
    "--output",
    path.join(fixture.root, "trusted-vrt-output"),
    "--threshold",
    "0",
    "--workers",
    "2",
  ];
  const announcement = new PublishedVrtAnnouncement(secrets);
  const decoder = new StringDecoder("utf8");
  let pending = "",
    stderrBytes = 0;
  let failure: Error | undefined,
    exit: { code: number | null; signal: NodeJS.Signals | null; stderrBytes: number } | undefined;
  let accept!: () => void, rejectReady!: (error: Error) => void;
  const ready = new Promise<void>((resolve, reject) => {
    accept = resolve;
    rejectReady = reject;
  });
  const child = spawn(process.execPath, args, {
    cwd: fixture.root,
    env: quickTunnelEnvironment(process.env),
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (child.pid) live.add(child.pid);
  let finish!: (value: NonNullable<typeof exit>) => void;
  const closed = new Promise<NonNullable<typeof exit>>((resolve) => {
    finish = resolve;
  });
  function refuse(message: string) {
    failure ??= new Error(message);
    rejectReady(failure);
    if (!exit) child.kill("SIGTERM");
  }
  child.once("error", () => refuse("Published VRT child startup failed"));
  child.once("close", (code, signal) => {
    exit = { code, signal, stderrBytes };
    pending = "";
    announcement.forget();
    if (child.pid) live.delete(child.pid);
    finish(exit);
    rejectReady(new Error("Published VRT child closed before listening"));
  });
  function consume(value: string) {
    if (failure || exit) return;
    pending += value;
    try {
      let split: number;
      while ((split = pending.indexOf("\n")) >= 0) {
        const row = pending.slice(0, split);
        pending = pending.slice(split + 1);
        if (announcement.line(row)) accept();
      }
      assert.ok(Buffer.byteLength(pending) <= 16_384);
    } catch {
      pending = "";
      refuse("Published VRT announcement contract refused");
    }
  }
  child.stdout.on("data", (bytes: Buffer) => {
    try {
      consume(decoder.write(bytes));
    } finally {
      bytes.fill(0);
    }
  });
  child.stdout.once("end", () => {
    consume(decoder.end());
    if (pending) consume("\n");
    pending = "";
  });
  child.stderr.on("data", (bytes: Buffer) => {
    stderrBytes += bytes.length;
    bytes.fill(0);
  });
  for (const stream of [child.stdout, child.stderr])
    stream.once("error", () => refuse("Published VRT pipe drainage failed"));
  async function wait(milliseconds: number) {
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
  async function close() {
    if (exit) return exit;
    child.kill("SIGTERM");
    const graceful = await wait(30_000);
    if (graceful) return graceful;
    child.kill("SIGKILL");
    const forced = await wait(5_000);
    assert.ok(forced, "Published VRT child/pipes did not close");
    return forced;
  }
  const timer = setTimeout(() => refuse("Published VRT readiness deadline expired"), 30_000);
  try {
    await ready;
    assert.ok(child.pid && !failure && !exit, "Published VRT readiness lost child ownership");
    const session = announcement.take();
    return {
      ...session,
      bin,
      args,
      pid: child.pid,
      assertRunning() {
        assert.ok(!failure && !exit, "Owned published VRT session stopped or refused");
      },
      close,
    };
  } catch (primary) {
    try {
      await close();
    } catch {
      throw new AggregateError(
        [primary, new Error("Published VRT cleanup failed")],
        "Published serve acquisition and cleanup refused",
      );
    }
    throw primary;
  } finally {
    clearTimeout(timer);
  }
}
