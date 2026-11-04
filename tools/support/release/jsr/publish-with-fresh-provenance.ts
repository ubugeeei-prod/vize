/**
 * Runs `deno publish` and replaces the final JSR provenance credential.
 *
 * Deno 2.9.7 (and 2.9.8) requests the permission-scoped GitHub OIDC token
 * before upload, then reuses it after polling. A publish of about ten minutes
 * expires that token. The separate Sigstore token is requested later and is
 * unaffected. GitHub's OIDC request API can mint another JSR token for this
 * same job; ask for one immediately before the provenance POST and do not
 * treat a registry rejection, or Deno's success warning, as acceptance.
 *
 * Deno appends `&audience=` to ACTIONS_ID_TOKEN_REQUEST_URL, so the rewritten
 * URL has to already contain `?`. This does not vendor Deno.
 */
import { spawn } from "node:child_process";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import type { AddressInfo } from "node:net";
import { pathToFileURL } from "node:url";
import type {
  JwtPayload,
  PermissionBinding,
  ProvenanceRoute,
  ProvenanceReceipt,
  PublishOptions,
  PublishResult,
} from "./provenance-types.ts";

const issuer = "https://token.actions.githubusercontent.com";
const minimumRemainingSeconds = 30;
const provenancePath = /^\/api\/scopes\/([^/]+)\/packages\/([^/]+)\/versions\/([^/]+)\/provenance$/;
const skippedHeaders = new Set([
  "host",
  "connection",
  "content-length",
  "transfer-encoding",
  "keep-alive",
]);
const identityClaims = [
  "sub",
  "actor",
  "actor_id",
  "repository",
  "repository_id",
  "repository_owner",
  "repository_owner_id",
  "repository_visibility",
  "workflow",
  "workflow_ref",
  "workflow_sha",
  "job_workflow_ref",
  "job_workflow_sha",
  "ref",
  "ref_type",
  "sha",
  "event_name",
  "run_id",
  "run_attempt",
  "run_number",
  "runner_environment",
  "environment",
  "check_run_id",
];

function segment(value: string): string | null {
  try {
    return decodeURIComponent(value);
  } catch {
    return null;
  }
}

/** The single `package/publish` binding Deno puts in the OIDC audience. */
export function parsePermissionAudience(audience: unknown): PermissionBinding | null {
  if (typeof audience !== "string" || audience.length === 0 || audience.length > 4096) return null;
  let parsed: Record<string, unknown> | null;
  try {
    parsed = JSON.parse(audience) as Record<string, unknown> | null;
  } catch {
    return null;
  }
  if (
    parsed === null ||
    typeof parsed !== "object" ||
    Array.isArray(parsed) ||
    Object.keys(parsed).length !== 1 ||
    !Array.isArray(parsed.permissions) ||
    parsed.permissions.length !== 1
  ) {
    return null;
  }
  const permission = parsed.permissions[0] as Record<string, unknown> | null;
  if (permission === null || typeof permission !== "object" || Array.isArray(permission))
    return null;
  const keys = Object.keys(permission).sort().join(",");
  if (keys !== "package,permission,scope,tarballHash,version") return null;
  const { permission: name, scope, package: packageName, version, tarballHash } = permission;
  if (name !== "package/publish") return null;
  if (
    typeof scope !== "string" ||
    typeof packageName !== "string" ||
    typeof version !== "string" ||
    typeof tarballHash !== "string"
  ) {
    return null;
  }
  if (
    scope.length === 0 ||
    packageName.length === 0 ||
    version.length === 0 ||
    !/^sha256-[0-9a-f]{64}$/.test(tarballHash)
  ) {
    return null;
  }
  return { scope, packageName, version, tarballHash };
}

function audienceClaim(payload: JwtPayload | null | undefined): string | null {
  const audience = payload?.aud;
  if (typeof audience === "string") return audience;
  if (Array.isArray(audience) && audience.length === 1 && typeof audience[0] === "string") {
    return audience[0];
  }
  return null;
}

export function decodeJwtPayload(token: unknown): JwtPayload | null {
  if (typeof token !== "string") return null;
  const parts = token.split(".");
  if (parts.length !== 3 || parts.some((part) => part.length === 0)) return null;
  try {
    const payload = JSON.parse(
      Buffer.from(parts[1], "base64url").toString("utf8"),
    ) as JwtPayload | null;
    if (payload === null || typeof payload !== "object" || Array.isArray(payload)) return null;
    return payload;
  } catch {
    return null;
  }
}

function githubOidc(header: unknown): string | null {
  if (typeof header !== "string") return null;
  const match = /^githuboidc (\S+)$/.exec(header);
  return match ? match[1] : null;
}

function sameIdentity(presented: JwtPayload, fresh: JwtPayload): boolean {
  if (presented.iss !== issuer || fresh.iss !== issuer) return false;
  return identityClaims.every(
    (claim) => JSON.stringify(presented[claim] ?? null) === JSON.stringify(fresh[claim] ?? null),
  );
}

/**
 * Accept a credential only when it is a new token for this job and the exact
 * permission audience captured from Deno's original OIDC request.
 */
export function freshCredentialFailure({
  presented,
  fresh,
  audience,
  now,
}: {
  presented: JwtPayload;
  fresh: JwtPayload;
  audience: string;
  now: number;
}): string | null {
  if (audienceClaim(presented) !== audience || audienceClaim(fresh) !== audience) {
    return "permission-audience";
  }
  const binding = parsePermissionAudience(audience);
  if (!binding) return "permission-audience";
  if (!sameIdentity(presented, fresh)) return "workflow-identity";
  if (
    typeof presented.jti !== "string" ||
    typeof fresh.jti !== "string" ||
    fresh.jti === presented.jti
  ) {
    return "not-fresh";
  }
  if (
    typeof presented.iat !== "number" ||
    typeof fresh.iat !== "number" ||
    typeof presented.exp !== "number" ||
    typeof fresh.exp !== "number" ||
    !(fresh.iat >= presented.iat) ||
    !(fresh.exp > presented.exp) ||
    !(fresh.exp > now + minimumRemainingSeconds)
  ) {
    return "not-fresh";
  }
  return null;
}

/** A zero exit is acceptance only when provenance was not rejected or skipped. */
export function exitCodeForPublish({
  childCode,
  output,
  receipt,
}: {
  childCode: number | null;
  output?: string;
  receipt?: Partial<ProvenanceReceipt> | null;
}): number {
  const text = output ?? "";
  if (receipt?.attempted === true && receipt.accepted !== true) return 1;
  if (/did not accept the provenance/i.test(text)) return 1;
  if (/Successfully published/.test(text) && receipt?.accepted !== true) return 1;
  return childCode === 0 ? 0 : (childCode ?? 1);
}

function readBody(request: IncomingMessage): Promise<Buffer<ArrayBuffer>> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    request.on("data", (chunk: Buffer) => chunks.push(chunk));
    request.on("end", () => resolve(Buffer.concat(chunks)));
    request.on("error", reject);
  });
}

function listen(server: Server): Promise<string> {
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address() as AddressInfo;
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
}

async function requestOidc(
  oidcUpstream: string,
  requestToken: string,
  audience: string,
): Promise<string | null> {
  const url = new URL(oidcUpstream);
  url.searchParams.set("audience", audience);
  const response = await fetch(url, {
    headers: { authorization: `Bearer ${requestToken}` },
  });
  const text = await response.text();
  if (!response.ok) return null;
  let parsed: { value?: unknown } | null;
  try {
    parsed = JSON.parse(text) as { value?: unknown } | null;
  } catch {
    return null;
  }
  return typeof parsed?.value === "string" && parsed.value.length > 0 ? parsed.value : null;
}

/**
 * @param {object} options
 * @param {readonly string[]} options.command
 * @param {string} [options.cwd]
 * @param {NodeJS.ProcessEnv} [options.env]
 * @param {string} [options.upstream]
 * @param {() => number} [options.now]
 * @param {boolean} [options.signals]
 */
export async function publishWithFreshProvenance(options: PublishOptions): Promise<PublishResult> {
  const command = options.command ?? [];
  if (command.length === 0) return { code: 2, receipt: emptyReceipt() };
  const env = options.env ?? process.env;
  const upstream = options.upstream ?? "https://jsr.io";
  const now = options.now ?? (() => Date.now() / 1000);
  const oidcUpstream = env.ACTIONS_ID_TOKEN_REQUEST_URL;
  const oidcRequestToken = env.ACTIONS_ID_TOKEN_REQUEST_TOKEN;
  const captured: { audience: string; binding: PermissionBinding }[] = [];
  const receipt = emptyReceipt();
  let output = "";

  const server = createServer((request, response) => {
    handle(request, response).catch(() => {
      if (!response.headersSent) response.writeHead(502, { "content-type": "text/plain" });
      response.end("JSR publish proxy failed");
    });
  });

  async function handle(request: IncomingMessage, response: ServerResponse): Promise<void> {
    const requestUrl = new URL(request.url ?? "/", "http://127.0.0.1");
    if (requestUrl.pathname === "/github-oidc") {
      const audience = requestUrl.searchParams.get("audience") ?? "";
      const binding = parsePermissionAudience(audience);
      if (binding) captured.push({ audience, binding });
      if (!oidcUpstream || !oidcRequestToken) {
        response.writeHead(500, { "content-type": "text/plain" }).end("missing OIDC request");
        return;
      }
      const url = new URL(oidcUpstream);
      if (audience) url.searchParams.set("audience", audience);
      const upstreamResponse = await fetch(url, {
        headers: { authorization: `Bearer ${oidcRequestToken}` },
      });
      const body = Buffer.from(await upstreamResponse.arrayBuffer());
      const headers: Record<string, string> = {};
      const contentType = upstreamResponse.headers.get("content-type");
      if (contentType) headers["content-type"] = contentType;
      response.writeHead(upstreamResponse.status, headers).end(body);
      return;
    }

    const body = await readBody(request);
    const parts = provenancePath.exec(requestUrl.pathname);
    const provenance =
      request.method === "POST" && parts
        ? {
            scope: segment(parts[1]),
            packageName: segment(parts[2]),
            version: segment(parts[3]),
          }
        : null;
    const headers = new Headers();
    for (const [name, value] of Object.entries(request.headers)) {
      if (skippedHeaders.has(name) || Array.isArray(value) || value === undefined) continue;
      headers.set(name, value);
    }
    if (provenance) {
      receipt.attempted = true;
      receipt.accepted = false;
      const failure = await authorizeProvenance(provenance, headers);
      if (failure) {
        receipt.reason = failure;
        response
          .writeHead(401, { "content-type": "application/json" })
          .end(JSON.stringify({ code: "invalidOidcToken", message: failure }));
        return;
      }
    }
    const target = new URL(
      requestUrl.pathname + requestUrl.search,
      upstream.endsWith("/") ? upstream : `${upstream}/`,
    );
    const upstreamResponse = await fetch(target, {
      method: request.method,
      headers,
      body: request.method === "GET" || request.method === "HEAD" ? undefined : body,
      redirect: "manual",
    });
    const upstreamBody = Buffer.from(await upstreamResponse.arrayBuffer());
    if (provenance) {
      receipt.attempted = true;
      receipt.registryStatus = upstreamResponse.status;
      receipt.accepted = upstreamResponse.ok;
      receipt.reason = upstreamResponse.ok ? null : "registry";
      const match = captured.find(
        (item) =>
          item.binding.scope === provenance.scope &&
          item.binding.packageName === provenance.packageName &&
          item.binding.version === provenance.version,
      );
      receipt.binding = match?.binding ?? null;
    }
    const responseHeaders: Record<string, string> = {};
    const contentType = upstreamResponse.headers.get("content-type");
    if (contentType) responseHeaders["content-type"] = contentType;
    response.writeHead(upstreamResponse.status, responseHeaders).end(upstreamBody);
  }

  async function authorizeProvenance(
    provenance: ProvenanceRoute,
    headers: Headers,
  ): Promise<string | null> {
    if (!provenance.scope || !provenance.packageName || !provenance.version)
      return "missing-binding";
    const match = captured.findLast(
      (item) =>
        item.binding.scope === provenance.scope &&
        item.binding.packageName === provenance.packageName &&
        item.binding.version === provenance.version,
    );
    if (!match) return "missing-binding";
    const presented = decodeJwtPayload(githubOidc(headers.get("authorization")));
    if (!presented) return "permission-audience";
    if (!oidcUpstream || !oidcRequestToken) return "not-fresh";
    const token = await requestOidc(oidcUpstream, oidcRequestToken, match.audience);
    const fresh = decodeJwtPayload(token);
    if (!fresh) return "not-fresh";
    const failure = freshCredentialFailure({
      presented,
      fresh,
      audience: match.audience,
      now: now(),
    });
    if (failure) return failure;
    headers.set("authorization", `githuboidc ${token}`);
    receipt.binding = match.binding;
    return null;
  }

  const local = await listen(server);
  const childEnv = {
    ...env,
    JSR_URL: local,
    ACTIONS_ID_TOKEN_REQUEST_URL: `${local}/github-oidc?`,
  };
  const child = spawn(command[0], command.slice(1), {
    cwd: options.cwd,
    env: childEnv,
    stdio: ["inherit", "pipe", "pipe"],
  });
  const onOutput = (chunk: Buffer, stream: NodeJS.WriteStream) => {
    output += chunk;
    stream.write(chunk);
  };
  child.stdout.on("data", (chunk) => onOutput(chunk, process.stdout));
  child.stderr.on("data", (chunk) => onOutput(chunk, process.stderr));
  let stopSignals = () => {};
  if (options.signals !== false) {
    const onSignal = () => {
      if (child.exitCode === null) child.kill("SIGINT");
      const code = exitCodeForPublish({ childCode: 124, output, receipt });
      process.exit(code === 124 ? 124 : 1);
    };
    process.on("SIGINT", onSignal);
    process.on("SIGTERM", onSignal);
    stopSignals = () => {
      process.off("SIGINT", onSignal);
      process.off("SIGTERM", onSignal);
    };
  }
  const childCode = await new Promise<number | null>((resolve) => {
    child.once("error", () => resolve(127));
    child.once("close", (code) => resolve(code));
  });
  stopSignals();
  server.closeAllConnections?.();
  await new Promise<void>((resolve) => server.close(() => resolve()));
  const code = exitCodeForPublish({ childCode, output, receipt });
  if (code === 1 && receipt.attempted && receipt.accepted !== true) {
    console.error(`JSR provenance was not accepted (${receipt.reason ?? "rejected"})`);
  } else if (code === 1 && /Successfully published|did not accept the provenance/i.test(output)) {
    console.error("JSR provenance was not accepted");
  }
  return { code, receipt };
}

function emptyReceipt(): ProvenanceReceipt {
  return {
    attempted: false,
    accepted: false,
    reason: null,
    registryStatus: null,
    binding: null,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const separator = process.argv.indexOf("--");
  const command = separator === -1 ? [] : process.argv.slice(separator + 1);
  if (command.length === 0) {
    console.error(
      "Usage: publish-with-fresh-provenance.ts -- <command...>\nExample: publish-with-fresh-provenance.ts -- deno publish --config jsr.json",
    );
    process.exit(2);
  }
  const result = await publishWithFreshProvenance({ command });
  process.exit(result.code);
}
