import assert from "node:assert/strict";
import { createServer, type Server } from "node:http";
import { test } from "node:test";

import {
  decodeJwtPayload,
  exitCodeForPublish,
  freshCredentialFailure,
  parsePermissionAudience,
  publishWithFreshProvenance,
} from "../../tools/support/release/jsr/publish-with-fresh-provenance.ts";

interface PermissionBinding {
  scope: string;
  packageName: string;
  version: string;
  tarballHash: string;
}
type JwtPayload = Readonly<Record<string, unknown>>;
interface OidcCall {
  audience: string | null;
  authorization: string | undefined;
}
interface RegistryCall {
  method: string | undefined;
  url: string | undefined;
  authorization: string | undefined;
  body: string;
}
interface ScenarioOptions {
  permissionAudience?: string;
  path?: string;
  freshAudience?: string | null;
  freshClaims?: JwtPayload;
  provenanceStatus?: number;
}

const now = 1_700_000_000;
const hash = `sha256-${"ab".repeat(32)}`;
const otherHash = `sha256-${"cd".repeat(32)}`;
const issuer = "https://token.actions.githubusercontent.com";
const versionPath = "/api/scopes/vizejs/packages/vize/versions/0.471.0";

const childSource = `
const audience = process.env.PERMISSION_AUDIENCE;
const versionPath = process.env.VERSION_PATH;
const registry = process.env.JSR_URL.replace(/\\/$/, "");
const oidcUrl = process.env.ACTIONS_ID_TOKEN_REQUEST_URL + "&audience=" + encodeURIComponent(audience);
const minted = await fetch(oidcUrl, {
  headers: { authorization: "Bearer " + process.env.ACTIONS_ID_TOKEN_REQUEST_TOKEN },
});
if (!minted.ok) throw new Error("oidc " + minted.status);
const { value } = await minted.json();
const authorization = "githuboidc " + value;
const upload = await fetch(registry + versionPath + "?config=/jsr.json", {
  method: "POST",
  headers: { authorization, "content-type": "application/gzip" },
  body: "tarball",
});
await upload.arrayBuffer();
if (!upload.ok) throw new Error("upload " + upload.status);
const provenance = await fetch(registry + versionPath + "/provenance", {
  method: "POST",
  headers: { authorization, "content-type": "application/json" },
  body: JSON.stringify({ bundle: { mediaType: "application/vnd.dev.sigstore.bundle+json" } }),
});
await provenance.arrayBuffer();
console.log("provenance-status=" + provenance.status);
`;

function audienceFor(binding: PermissionBinding) {
  return JSON.stringify({
    permissions: [
      {
        permission: "package/publish",
        scope: binding.scope,
        package: binding.packageName,
        version: binding.version,
        tarballHash: binding.tarballHash,
      },
    ],
  });
}

const binding = {
  scope: "vizejs",
  packageName: "vize",
  version: "0.471.0",
  tarballHash: hash,
};
const audience = audienceFor(binding);

function claims(tokenAudience: string | null, overrides: JwtPayload = {}) {
  const payload = {
    iss: issuer,
    sub: "repo:ubugeeei-prod/vize:ref:refs/heads/main",
    repository: "ubugeeei-prod/vize",
    repository_id: "869000001",
    repository_owner: "ubugeeei-prod",
    repository_owner_id: "869000002",
    repository_visibility: "public",
    workflow: "Release JSR",
    workflow_ref: "ubugeeei-prod/vize/.github/workflows/release-jsr.yml@refs/heads/main",
    workflow_sha: "5d9f80f824ae27232de9dc84246f7e5eff36594f",
    ref: "refs/heads/main",
    ref_type: "branch",
    sha: "5d9f80f824ae27232de9dc84246f7e5eff36594f",
    event_name: "workflow_dispatch",
    run_id: "39143153411",
    run_attempt: "1",
    run_number: "470",
    actor: "ubugeeei",
    actor_id: "1",
    runner_environment: "github-hosted",
    ...overrides,
    aud: tokenAudience,
  };
  payload.iss = issuer;
  return payload;
}

function jwt(payload: JwtPayload) {
  const encode = (value: JwtPayload) => Buffer.from(JSON.stringify(value)).toString("base64url");
  return `${encode({ alg: "none", typ: "JWT" })}.${encode(payload)}.sig`;
}

function payloadOf(header: string | undefined) {
  const token = header?.replace(/^githuboidc /, "") ?? "";
  return decodeJwtPayload(token);
}

function listen(server: Server): Promise<string> {
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("missing port"));
        return;
      }
      resolve(`http://127.0.0.1:${address.port}`);
    });
  });
}

function close(server: Server): Promise<void> {
  server.closeAllConnections?.();
  return new Promise((resolve) => server.close(() => resolve()));
}

async function runScenario({
  permissionAudience = audience,
  path = versionPath,
  freshAudience = null,
  freshClaims = { jti: "fresh", iat: now, exp: now + 300 },
  provenanceStatus = 204,
}: ScenarioOptions) {
  const oidcCalls: OidcCall[] = [];
  const jsrCalls: RegistryCall[] = [];
  const oidc = createServer((request, response) => {
    request.resume();
    const url = new URL(request.url ?? "/", "http://127.0.0.1");
    const requested = url.searchParams.get("audience");
    oidcCalls.push({ audience: requested, authorization: request.headers.authorization });
    const tokenAudience = oidcCalls.length === 1 ? requested : (freshAudience ?? requested);
    const payload =
      oidcCalls.length === 1
        ? claims(tokenAudience, { jti: "initial", iat: now - 600, nbf: now - 600, exp: now - 60 })
        : claims(tokenAudience, freshClaims);
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ value: jwt(payload) }));
  });
  const registry = createServer(async (request, response) => {
    const chunks: Buffer[] = [];
    for await (const chunk of request) chunks.push(chunk);
    jsrCalls.push({
      method: request.method,
      url: request.url,
      authorization: request.headers.authorization,
      body: Buffer.concat(chunks).toString("utf8"),
    });
    if (request.url?.split("?")[0]?.endsWith("/provenance")) {
      response.writeHead(provenanceStatus, { "content-type": "application/json" });
      response.end(
        provenanceStatus === 204
          ? ""
          : JSON.stringify({ code: "invalidOidcToken", message: "ExpiredSignature" }),
      );
      return;
    }
    response.writeHead(201, { "content-type": "application/json" });
    response.end(JSON.stringify({ id: "task", status: "success" }));
  });
  const oidcUrl = await listen(oidc);
  const registryUrl = await listen(registry);
  try {
    const result = await publishWithFreshProvenance({
      command: [process.execPath, "--input-type=module", "-e", childSource],
      env: {
        ...process.env,
        ACTIONS_ID_TOKEN_REQUEST_URL: oidcUrl,
        ACTIONS_ID_TOKEN_REQUEST_TOKEN: "job-request-token",
        PERMISSION_AUDIENCE: permissionAudience,
        VERSION_PATH: path,
      },
      upstream: registryUrl,
      now: () => now,
      signals: false,
    });
    return { ...result, oidcCalls, jsrCalls };
  } finally {
    await Promise.all([close(oidc), close(registry)]);
  }
}

function provenanceCall(jsrCalls: readonly RegistryCall[]) {
  return jsrCalls.find((call) => call.url?.split("?")[0]?.endsWith("/provenance"));
}

await test("the final provenance POST uses a fresh token for the original permission binding", async () => {
  const result = await runScenario({});
  assert.equal(result.code, 0);
  assert.equal(result.receipt.accepted, true);
  assert.equal(result.receipt.registryStatus, 204);
  assert.deepEqual(result.receipt.binding, binding);
  assert.equal(result.oidcCalls.length, 2);
  assert.equal(result.oidcCalls[0].audience, audience);
  assert.equal(result.oidcCalls[1].audience, audience);
  assert.equal(result.oidcCalls[0].authorization, "Bearer job-request-token");
  assert.equal(result.oidcCalls[1].authorization, "Bearer job-request-token");
  const upload = result.jsrCalls.find((call) => call.url?.includes("config=/jsr.json"));
  const provenance = provenanceCall(result.jsrCalls);
  const initial = payloadOf(upload?.authorization);
  const fresh = payloadOf(provenance?.authorization);
  assert.equal(initial?.jti, "initial");
  assert.equal(initial?.exp, now - 60);
  assert.equal(fresh?.jti, "fresh");
  assert.equal(fresh?.aud, audience);
  assert.equal(fresh?.run_id, initial?.run_id);
  assert.equal(fresh?.workflow_ref, initial?.workflow_ref);
  assert.notEqual(provenance?.authorization, upload?.authorization);
  assert.equal(
    provenance?.body,
    JSON.stringify({ bundle: { mediaType: "application/vnd.dev.sigstore.bundle+json" } }),
  );
});

await test("negative permission bindings are not forwarded on the provenance POST", async () => {
  const cases = [
    { name: "tarball hash", freshAudience: audienceFor({ ...binding, tarballHash: otherHash }) },
    { name: "package", freshAudience: audienceFor({ ...binding, packageName: "other" }) },
    { name: "version", freshAudience: audienceFor({ ...binding, version: "0.471.1" }) },
    { name: "scope", freshAudience: audienceFor({ ...binding, scope: "other" }) },
  ];
  for (const item of cases) {
    const result = await runScenario({ freshAudience: item.freshAudience });
    assert.equal(result.code, 1, item.name);
    assert.equal(result.receipt.accepted, false, item.name);
    assert.equal(result.receipt.reason, "permission-audience", item.name);
    assert.equal(result.oidcCalls[1]?.audience, audience, item.name);
    assert.equal(provenanceCall(result.jsrCalls), undefined, item.name);
    assert.equal(payloadOf(result.jsrCalls[0]?.authorization)?.jti, "initial", item.name);
  }
});

await test("a fresh token for a different workflow job is rejected", async () => {
  const result = await runScenario({
    freshClaims: { jti: "fresh", iat: now, nbf: now, exp: now + 300, run_id: "999" },
  });
  assert.equal(result.code, 1);
  assert.equal(result.receipt.reason, "workflow-identity");
  assert.equal(provenanceCall(result.jsrCalls), undefined);
  assert.equal(result.oidcCalls[1]?.audience, audience);
});

await test("an audience captured for another version cannot authorize this provenance POST", async () => {
  const result = await runScenario({
    path: "/api/scopes/vizejs/packages/vize/versions/0.471.1",
  });
  assert.equal(result.code, 1);
  assert.equal(result.receipt.reason, "missing-binding");
  assert.equal(result.oidcCalls.length, 1);
  assert.equal(provenanceCall(result.jsrCalls), undefined);
});

await test("a replayed or still-expired credential is not a fresh token", async () => {
  const replayed = await runScenario({
    freshClaims: { jti: "initial", iat: now, nbf: now, exp: now + 300 },
  });
  assert.equal(replayed.code, 1);
  assert.equal(replayed.receipt.reason, "not-fresh");
  assert.equal(provenanceCall(replayed.jsrCalls), undefined);
  const expired = await runScenario({
    freshClaims: { jti: "fresh-expired", iat: now - 120, nbf: now - 120, exp: now - 1 },
  });
  assert.equal(expired.code, 1);
  assert.equal(expired.receipt.reason, "not-fresh");
  assert.equal(provenanceCall(expired.jsrCalls), undefined);
});

await test("registry rejection of a fresh credential fails the publish", async () => {
  const result = await runScenario({ provenanceStatus: 400 });
  assert.equal(result.code, 1);
  assert.equal(result.receipt.accepted, false);
  assert.equal(result.receipt.reason, "registry");
  assert.equal(result.receipt.registryStatus, 400);
  const provenance = provenanceCall(result.jsrCalls);
  assert.equal(payloadOf(provenance?.authorization)?.jti, "fresh");
  assert.equal(payloadOf(provenance?.authorization)?.aud, audience);
  assert.notEqual(payloadOf(provenance?.authorization)?.exp, now - 60);
});

await test("Deno's successful provenance warning is not acceptance", () => {
  const warning = [
    "Warning: the registry did not accept the provenance attestation: ExpiredSignature (invalidOidcToken)",
    "The package was published, but it will not show a provenance badge.",
    "Successfully published @vizejs/vize@0.471.0",
  ].join("\n");
  assert.equal(
    exitCodeForPublish({
      childCode: 0,
      output: warning,
      receipt: {
        attempted: false,
        accepted: false,
        reason: null,
        registryStatus: null,
        binding: null,
      },
    }),
    1,
  );
  assert.equal(
    exitCodeForPublish({
      childCode: 0,
      output: "Successfully published @vizejs/vize@0.471.0",
      receipt: {
        attempted: false,
        accepted: false,
        reason: null,
        registryStatus: null,
        binding: null,
      },
    }),
    1,
  );
  assert.equal(
    exitCodeForPublish({
      childCode: 0,
      output: "Warning: Skipping, already published @vizejs/vize@0.471.0",
      receipt: {
        attempted: false,
        accepted: false,
        reason: null,
        registryStatus: null,
        binding: null,
      },
    }),
    0,
  );
  assert.equal(
    exitCodeForPublish({
      childCode: 124,
      output: "",
      receipt: {
        attempted: false,
        accepted: false,
        reason: null,
        registryStatus: null,
        binding: null,
      },
    }),
    124,
  );
});

await test("permission audiences that are not one package publish binding are rejected", () => {
  assert.equal(parsePermissionAudience("sigstore"), null);
  assert.equal(
    parsePermissionAudience(audienceFor({ ...binding, tarballHash: "ab".repeat(32) })),
    null,
  );
  const widened = JSON.stringify({
    permissions: [
      {
        permission: "package/publish",
        scope: "vizejs",
        package: "vize",
        version: "0.471.0",
        tarballHash: hash,
      },
      {
        permission: "package/publish",
        scope: "vizejs",
        package: "other",
        version: "0.471.0",
        tarballHash: hash,
      },
    ],
  });
  assert.equal(parsePermissionAudience(widened), null);
  assert.deepEqual(parsePermissionAudience(audience), binding);
  const presented = claims(audience, { jti: "initial", iat: now - 600, exp: now - 60 });
  const fresh = claims(audienceFor({ ...binding, tarballHash: otherHash }), {
    jti: "fresh",
    iat: now,
    exp: now + 300,
  });
  assert.equal(freshCredentialFailure({ presented, fresh, audience, now }), "permission-audience");
});
