import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import type { Browser, BrowserContext, CDPSession, Page } from "playwright";
import type { Evidence } from "./vrt_artifacts.ts";
import type { VrtFixture } from "./vrt_fixtures.ts";
import { hostedPath, staticVrtHost } from "./vrt_host.ts";
import type { VrtPhase, VrtSecrets } from "./vrt_secrets.ts";
import { startPublishedVrtSession } from "./vrt_service_process.ts";
import { writeQuickTunnelConfig } from "./vrt_tunnel.ts";
import { startStaticQuickTunnel } from "./vrt_tunnel_process.ts";

const unclosed = new Set<object>();
export function assertTrustedVrtSurfacesClosed(): void {
  assert.equal(unclosed.size, 0, "Trusted VRT acquisition/cleanup custody incomplete");
}

interface Surface {
  page: Page;
  cdp: CDPSession;
  galleryUrl: string;
  session: Awaited<ReturnType<typeof startPublishedVrtSession>>;
  assertRunning(): void;
}

/** Unexecuted preparation: same supplied Browser; no browser launch, trust or security flags. */
export async function withTrustedVrtSurface(
  browser: Browser,
  input: {
    binary: string;
    assetDirectory: string;
    directory: string;
    fixture: Pick<VrtFixture, "root" | "config">;
    evidence: Evidence;
    secrets: VrtSecrets;
  },
  observe: (surface: Surface) => Promise<void>,
): Promise<void> {
  const ownership = {};
  unclosed.add(ownership);
  const owned: {
    task?: string;
    host?: Awaited<ReturnType<typeof staticVrtHost>>;
    tunnel?: Awaited<ReturnType<typeof startStaticQuickTunnel>>;
    session?: Awaited<ReturnType<typeof startPublishedVrtSession>>;
    context?: BrowserContext;
    cdp?: CDPSession;
  } = {};
  const acquisitionPhases = {
    "private-config-task": "trusted-config-task",
    "emitted-static-host": "trusted-static-host",
    "static-only-pinned-tunnel": "trusted-tunnel",
    "published-loopback-session": "trusted-session",
    "same-browser-context": "trusted-browser-context",
    "granular-permission-CDP": "trusted-cdp",
  } as const satisfies Record<string, VrtPhase>;
  const cleanupPhases = {
    "granular-permission-CDP": "trusted-cleanup-cdp",
    "same-browser-context": "trusted-cleanup-context",
    "published-loopback-session": "trusted-cleanup-session",
    "static-only-pinned-tunnel": "trusted-cleanup-tunnel",
    "emitted-static-host": "trusted-cleanup-host",
    "owned-empty-config": "trusted-cleanup-config",
  } as const satisfies Record<string, VrtPhase>;
  let acquiring: keyof typeof acquisitionPhases | undefined;
  let failed = false;
  let failure: unknown;
  const cleanupErrors: Error[] = [];
  // Register every release before the first acquisition. All releases are attempted.
  const releases = [
    { name: "granular-permission-CDP", run: async () => owned.cdp?.detach() },
    { name: "same-browser-context", run: async () => owned.context?.close() },
    {
      name: "published-loopback-session",
      run: async () => {
        if (!owned.session) return;
        const exit = await owned.session.close();
        assert.equal(exit.code, 0);
        assert.equal(exit.signal, null);
        assert.equal(exit.stderrBytes, 0);
      },
    },
    {
      name: "static-only-pinned-tunnel",
      run: async () => {
        if (!owned.tunnel) return;
        const exit = await owned.tunnel.close();
        assert.equal(exit.code, 0);
        assert.equal(exit.signal, null);
        assert.equal(exit.category, undefined);
      },
    },
    { name: "emitted-static-host", run: async () => owned.host?.close() },
    {
      name: "owned-empty-config",
      run: async () => {
        if (owned.task) await rm(owned.task, { recursive: true });
      },
    },
  ] as const;
  try {
    acquiring = "private-config-task";
    owned.task = await mkdtemp(path.join(os.tmpdir(), "vize-vrt-trusted-"));
    acquiring = undefined;
    const configDirectory = path.join(owned.task, "config");
    const config = await writeQuickTunnelConfig(configDirectory);
    acquiring = "emitted-static-host";
    owned.host = await staticVrtHost(input.directory, input.evidence);
    acquiring = undefined;
    const staticOrigin = new URL(owned.host.url).origin;
    acquiring = "static-only-pinned-tunnel";
    owned.tunnel = await startStaticQuickTunnel({
      binary: input.binary,
      assetDirectory: input.assetDirectory,
      config,
      configDirectory,
      staticOrigin,
    });
    acquiring = undefined;
    const galleryUrl = `${owned.tunnel.origin}${hostedPath}`;
    owned.tunnel.assertRunning();
    acquiring = "published-loopback-session";
    owned.session = await startPublishedVrtSession(input.fixture, galleryUrl, input.secrets);
    acquiring = undefined;
    owned.tunnel.assertRunning();
    owned.session.assertRunning();
    acquiring = "same-browser-context";
    owned.context = await browser.newContext();
    acquiring = undefined;
    const page = await owned.context.newPage();
    acquiring = "granular-permission-CDP";
    owned.cdp = await owned.context.newCDPSession(page);
    acquiring = undefined;
    const tunnel = owned.tunnel;
    const session = owned.session;
    const assertRunning = () => {
      assert.ok(browser.isConnected(), "Original public Browser disconnected");
      tunnel.assertRunning();
      session.assertRunning();
    };
    assertRunning();
    await observe({ page, cdp: owned.cdp, galleryUrl, session, assertRunning });
    assertRunning();
  } catch (error) {
    input.secrets.failed(acquiring ? acquisitionPhases[acquiring] : "trusted-observation");
    failed = true;
    failure = input.secrets.sanitizeError(error);
  } finally {
    for (const release of releases) {
      try {
        await release.run();
      } catch {
        input.secrets.failed(cleanupPhases[release.name]);
        cleanupErrors.push(new Error(`Owned ${release.name} cleanup refused`));
      }
    }
    // A rejected acquisition has no returned close receipt: conservatively refuse the barrier.
    if (!acquiring && !cleanupErrors.length) unclosed.delete(ownership);
  }
  if (failed || cleanupErrors.length)
    throw new AggregateError(
      [...(failed ? [failure] : []), ...cleanupErrors],
      "Trusted gallery observation/acquisition/cleanup refused",
    );
}
