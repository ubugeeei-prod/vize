import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { createHash, X509Certificate } from "node:crypto";
import { mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:https";
import os from "node:os";
import path from "node:path";
import * as tls from "node:tls";
import { promisify } from "node:util";
import { chromium, type Browser, type Page } from "playwright";

const execute = promisify(execFile);
const buildPrefix = "/built/";
const galleryPrefix = "/built/gallery/";
const contentTypes: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
};

export interface SecureHost {
  origin: string;
  spki: string;
  certificatePath: string;
  /** Change the served physical build without intercepting browser requests. */
  directory: string;
  /** Response redirects, keyed by pathname. No response bodies are rewritten. */
  redirects: Map<string, string>;
  /** Deliberately omit query strings, request headers and credentials. */
  requests: Array<{ method: string; pathname: string; status: number }>;
  close(): Promise<void>;
}

/** Plain static bytes, rewritten only from the mounted gallery URL to its physical build. */
export async function createSecureHost(directory: string): Promise<SecureHost> {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "musea-hosted-network-"));
  const certificatePath = path.join(temporary, "certificate.pem");
  const keyPath = path.join(temporary, "key.pem");
  const configPath = path.join(temporary, "openssl.cnf");
  await writeFile(
    configPath,
    `[req]\nprompt = no\ndistinguished_name = dn\nx509_extensions = extensions\n` +
      `[dn]\nCN = 127.0.0.1\n[extensions]\nsubjectAltName = IP:127.0.0.1\n` +
      `basicConstraints = critical,CA:TRUE\nkeyUsage = critical,digitalSignature,keyEncipherment,keyCertSign\n` +
      `extendedKeyUsage = serverAuth\n`,
  );
  let host: SecureHost;
  try {
    await execute("openssl", [
      "req",
      "-x509",
      "-newkey",
      "rsa:2048",
      "-sha256",
      "-nodes",
      "-days",
      "1",
      "-config",
      configPath,
      "-keyout",
      keyPath,
      "-out",
      certificatePath,
    ]);
    const certificate = await readFile(certificatePath);
    const parsed = new X509Certificate(certificate);
    assert.equal(parsed.checkIP("127.0.0.1"), "127.0.0.1");
    const spki = createHash("sha256")
      .update(parsed.publicKey.export({ type: "spki", format: "der" }))
      .digest("base64");
    const server = createServer({ cert: certificate, key: await readFile(keyPath) }, (req, res) => {
      const pathname = new URL(req.url ?? "/", "https://127.0.0.1").pathname;
      res.setHeader("Cache-Control", "no-store");
      res.once("finish", () => {
        host.requests.push({ method: req.method ?? "GET", pathname, status: res.statusCode });
      });
      void (async () => {
        const redirect = host.redirects.get(pathname);
        if (redirect !== undefined) {
          res.writeHead(302, { Location: redirect }).end();
          return;
        }
        if (pathname === galleryPrefix.slice(0, -1)) {
          res.writeHead(308, { Location: galleryPrefix }).end();
          return;
        }
        if (!pathname.startsWith(buildPrefix)) {
          res.writeHead(404).end("Not found");
          return;
        }
        let relative = decodeURIComponent(pathname.slice(buildPrefix.length));
        if (!relative || relative.endsWith("/")) relative += "index.html";
        const root = await realpath(host.directory);
        const target = path.resolve(root, relative);
        if (!target.startsWith(`${root}${path.sep}`)) {
          res.writeHead(403).end("Forbidden");
          return;
        }
        let filename: string;
        let body: Buffer;
        try {
          filename = await realpath(target);
          if (!filename.startsWith(`${root}${path.sep}`)) {
            res.writeHead(403).end("Forbidden");
            return;
          }
          body = await readFile(filename);
        } catch (error) {
          if (!pathname.startsWith(galleryPrefix) || !req.headers.accept?.includes("text/html")) {
            throw error;
          }
          filename = await realpath(path.join(root, "gallery/index.html"));
          if (!filename.startsWith(`${root}${path.sep}`)) {
            res.writeHead(403).end("Forbidden");
            return;
          }
          body = await readFile(filename);
        }
        res.setHeader(
          "Content-Type",
          contentTypes[path.extname(filename)] ?? "application/octet-stream",
        );
        res.writeHead(200).end(req.method === "HEAD" ? undefined : body);
      })().catch(() => {
        if (!res.headersSent) res.writeHead(404).end("Not found");
        else res.destroy();
      });
    });
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    host = {
      origin: `https://127.0.0.1:${address.port}`,
      spki,
      certificatePath,
      directory,
      redirects: new Map(),
      requests: [],
      async close() {
        try {
          await new Promise<void>((resolve, reject) => {
            server.close((error) => (error ? reject(error) : resolve()));
            server.closeAllConnections();
          });
        } finally {
          await rm(temporary, { recursive: true, force: true });
        }
      },
    };
    return host;
  } catch (error) {
    await rm(temporary, { recursive: true, force: true });
    throw error;
  }
}

/** Only this certificate is trusted; only this server is classified as public. */
export function launchPublicBrowser(host: SecureHost): Promise<Browser> {
  const address = new URL(host.origin);
  return chromium.launch({
    args: [
      `--ignore-certificate-errors-spki-list=${host.spki}`,
      `--ip-address-space-overrides=127.0.0.1:${address.port}=public`,
    ],
  });
}

/** Keep the CDP session attached through requests; detaching resets its permission override. */
export async function setLoopbackPermission(
  page: Page,
  setting: "granted" | "denied",
): Promise<() => Promise<void>> {
  const origin = new URL(page.url()).origin;
  assert.ok(origin.startsWith("https://"));
  const session = await page.context().newCDPSession(page);
  try {
    const { targetInfo } = await session.send("Target.getTargetInfo");
    assert.ok(targetInfo.browserContextId);
    await session.send("Browser.setPermission", {
      permission: { name: "loopback-network" },
      setting,
      origin,
      browserContextId: targetInfo.browserContextId,
    });
    const observed = await page.evaluate(async () => {
      const descriptor = { name: "loopback-network" } as PermissionDescriptor;
      return (await navigator.permissions.query(descriptor)).state;
    });
    assert.equal(observed, setting);
    return () => session.detach();
  } catch (error) {
    await session.detach();
    throw error;
  }
}

/** Node's CA seam is current-thread only. Always call the returned cleanup function. */
export async function trustSecureHost(host: SecureHost): Promise<() => void> {
  const original = tls.getCACertificates("default");
  const certificate = await readFile(host.certificatePath, "utf8");
  tls.setDefaultCACertificates([...original, certificate]);
  return () => tls.setDefaultCACertificates(original);
}

export interface AddressSpaceObservation {
  kind: "request" | "response";
  origin?: string;
  pathname?: string;
  initiatorIsSecureContext?: boolean;
  initiatorIPAddressSpace?: string;
  resourceIPAddressSpace?: string;
  localNetworkAccessRequestPolicy?: string;
}

/** Actual CDP evidence: retain only URL origins/pathnames and address-space classifications. */
export async function createAddressSpaceObserver(page: Page): Promise<{
  records: AddressSpaceObservation[];
  close(): Promise<void>;
}> {
  const session = await page.context().newCDPSession(page);
  const records: AddressSpaceObservation[] = [];
  const urls = new Map<string, { origin: string; pathname: string }>();
  const pending = new Map<string, AddressSpaceObservation[]>();
  const record = (requestId: string, observation: AddressSpaceObservation) => {
    const url = urls.get(requestId);
    if (url) Object.assign(observation, url);
    else pending.set(requestId, [...(pending.get(requestId) ?? []), observation]);
    records.push(observation);
  };
  session.on("Network.requestWillBeSent", ({ requestId, request }) => {
    const url = new URL(request.url);
    const sanitized = { origin: url.origin, pathname: url.pathname };
    urls.set(requestId, sanitized);
    for (const observation of pending.get(requestId) ?? []) Object.assign(observation, sanitized);
    pending.delete(requestId);
  });
  session.on("Network.requestWillBeSentExtraInfo", ({ requestId, clientSecurityState }) => {
    if (!clientSecurityState) return;
    record(requestId, {
      kind: "request",
      initiatorIsSecureContext: clientSecurityState.initiatorIsSecureContext,
      initiatorIPAddressSpace: clientSecurityState.initiatorIPAddressSpace,
      localNetworkAccessRequestPolicy: clientSecurityState.localNetworkAccessRequestPolicy,
    });
  });
  session.on("Network.responseReceivedExtraInfo", ({ requestId, resourceIPAddressSpace }) => {
    record(requestId, { kind: "response", resourceIPAddressSpace });
  });
  try {
    await session.send("Network.enable");
    return { records, close: () => session.detach() };
  } catch (error) {
    await session.detach();
    throw error;
  }
}
