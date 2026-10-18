import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  assertPublisher,
  crates,
  namespaceResponse,
  positive,
  record,
  type Crate,
} from "./policy.ts";

// Schema pinned to rust-lang/crates.io@33e1ca9abb94e17779d580bd37516c6db1bcd1ab:
// src/controllers/trustpub/github_configs/{list,create,json}.rs and
// crates/crates_io_api_types/src/trustpub.rs. GET /me is cookie-only.
export const registryOrigin = "https://crates.io";
export const configPath = "/api/v1/trusted_publishing/github_configs";
export type Request = (url: string, init?: RequestInit) => Promise<Response>;
export const sha256 = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex");
const headers = { "User-Agent": "vize-corsa-first-publication (github.com/ubugeeei-prod/vize)" };

export function boundedRequest(deadline: number, implementation: Request = fetch): Request {
  return async (url, init = {}) => {
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, "original bootstrap deadline expired");
    const merged = new Headers(headers);
    new Headers(init.headers).forEach((value, key) => merged.set(key, value));
    const response = await implementation(url, {
      ...init,
      redirect: "manual",
      headers: merged,
      signal: AbortSignal.any([
        ...(init.signal ? [init.signal] : []),
        AbortSignal.timeout(Math.min(30_000, remaining)),
      ]),
    });
    assert.ok(deadline > Date.now(), "original bootstrap deadline expired during registry request");
    return response;
  };
}
export async function namespace(name: Crate, request: Request) {
  const response = await request(`${registryOrigin}/api/v1/crates/${name}`);
  return namespaceResponse(name, response.status, await response.json());
}
function authenticated(token: string) {
  assert.ok(token.length > 0 && !/[\r\n]/u.test(token), "explicit protected credential required");
  return { Authorization: token, "Content-Type": "application/json" };
}
export async function configs(
  selector: { crate: Crate } | { user_id: number },
  token: string,
  request: Request,
) {
  const [key, value] = Object.entries(selector)[0];
  if (key === "user_id") positive(value, "intended crates.io user ID");
  const first = new URL(configPath, registryOrigin);
  first.searchParams.set(key, String(value));
  first.searchParams.set("per_page", "100");
  let next: URL | undefined = first;
  const seen = new Set<string>();
  const rows: Record<string, unknown>[] = [];
  while (next) {
    assert.ok(!seen.has(next.href) && seen.size < 20, "invalid config pagination");
    seen.add(next.href);
    const response = await request(next.href, { headers: authenticated(token) });
    assert.equal(response.status, 200, "authenticated Trusted Publishing inventory failed");
    const body = record(await response.json());
    assert.ok(Array.isArray(body.github_configs), "github_configs array required");
    rows.push(...body.github_configs.map(record));
    const meta = record(body.meta);
    assert.ok(Number.isSafeInteger(meta.total) && Number(meta.total) >= 0);
    if (meta.next_page === null) {
      assert.equal(rows.length, meta.total, "incomplete Trusted Publishing inventory");
      next = undefined;
    } else {
      assert.ok(typeof meta.next_page === "string");
      const page = new URL(meta.next_page, first);
      assert.equal(page.origin, registryOrigin);
      assert.equal(page.pathname, configPath);
      assert.equal(
        page.searchParams.get(key),
        String(value),
        "pagination changed authority selector",
      );
      assert.equal(page.searchParams.has(key === "crate" ? "user_id" : "crate"), false);
      assert.equal(page.searchParams.has("page"), false, "server uses seek pagination");
      assert.ok(page.searchParams.has("seek"), "pagination seek required");
      next = page;
    }
  }
  assert.equal(new Set(rows.map((row) => row.id)).size, rows.length, "duplicate config records");
  return rows;
}

/** This endpoint authenticates the requested user ID, even before names exist. */
export async function intendedOwner(owner: number, token: string, request: Request) {
  positive(owner, "intended crates.io user ID");
  await configs({ user_id: owner }, token, request);
}

/** Bind caller input to the real direct maintainer of the canonical package. */
export async function canonicalOwner(
  owner: number,
  login: string,
  request: Request,
  name = "vize",
) {
  positive(owner, "canonical owner ID");
  assert.match(login, /^[a-zA-Z0-9-]+$/u);
  assert.ok(name === "vize" || crates.includes(name as Crate));
  const response = await request(`${registryOrigin}/api/v1/crates/${name}/owners`);
  assert.equal(response.status, 200, "live direct-owner inventory required");
  const rows = record(await response.json()).users;
  assert.ok(Array.isArray(rows));
  const matches = rows
    .map(record)
    .filter(
      (row) =>
        row.id === owner &&
        row.login === login &&
        row.kind === "user" &&
        row.url === `https://github.com/${login}`,
    );
  assert.equal(matches.length, 1, "caller is not the actual canonical direct maintainer");
  return matches[0];
}
export async function ensurePublisher(
  name: Crate,
  ownerId: number,
  token: string,
  request: Request,
  beforeWrite: () => void | Promise<void>,
) {
  let rows = await configs({ crate: name }, token, request);
  // A token must have direct user ownership for this list. Unknown existing trust
  // is not silently adopted, deleted, or augmented by a first-publication helper.
  assert.ok(rows.length <= 1, "unexpected existing publisher inventory; inspect manually");
  if (rows.length === 0) {
    await beforeWrite();
    const response = await request(`${registryOrigin}${configPath}`, {
      method: "POST",
      headers: authenticated(token),
      body: JSON.stringify({
        github_config: {
          crate: name,
          repository_owner: "ubugeeei-prod",
          repository_name: "vize",
          workflow_filename: "release.yml",
          environment: "crates-io",
        },
      }),
    });
    assert.equal(
      response.status,
      200,
      "Trusted Publishing creation failed; preserve partial custody",
    );
    assertPublisher(record(await response.json()).github_config, name, ownerId);
    rows = await configs({ crate: name }, token, request);
  }
  assert.equal(rows.length, 1, "exactly one server-verified publisher required");
  return assertPublisher(rows[0], name, ownerId);
}
export async function exactVersion(name: Crate, version: string, request: Request) {
  assert.ok(crates.includes(name));
  const response = await request(`${registryOrigin}/api/v1/crates/${name}/${version}`);
  assert.equal(response.status, 200, `exact published version missing: ${name}@${version}`);
  const row = record(record(await response.json()).version);
  assert.equal(row.crate, name);
  assert.equal(row.num, version);
  assert.equal(row.yanked, false);
  assert.match(String(row.checksum), /^[0-9a-f]{64}$/u);
  return row.checksum as string;
}

export async function archive(name: Crate, version: string, checksum: string, request: Request) {
  const response = await request(`${registryOrigin}/api/v1/crates/${name}/${version}/download`);
  assert.ok(
    response.status === 302 || response.status === 307,
    "official archive redirect required",
  );
  const location = response.headers.get("location");
  assert.equal(location, `https://static.crates.io/crates/${name}/${name}-${version}.crate`);
  const download = await request(location);
  assert.equal(download.status, 200);
  const bytes = Buffer.from(await download.arrayBuffer());
  assert.equal(sha256(bytes), checksum, "downloaded archive differs from crates.io checksum");
  return bytes;
}
