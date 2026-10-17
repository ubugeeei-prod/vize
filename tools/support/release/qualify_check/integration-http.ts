import assert from "node:assert/strict";
import { githubApiPages, githubApiRequest } from "../../compat/github/release-preflight-github.mjs";
import type { ReadApi } from "./check-state.ts";

type HttpOptions = {
  repository: string;
  token: string;
  apiUrl?: string;
  fetchImpl?: typeof fetch;
  now?: () => number;
  sleep?: (milliseconds: number) => Promise<void>;
};

/** Read-only transport; only an authenticated pending Q may latch its original deadline. */
export function createIntegrationReadApi(options: HttpOptions) {
  const now = options.now ?? Date.now;
  const pause =
    options.sleep ?? ((ms: number) => new Promise<void>((done) => setTimeout(done, ms)));
  let deadline: number | undefined;
  const remaining = () => {
    const value = deadline! - now();
    assert.ok(Number.isSafeInteger(value) && value > 0, "original Q transport deadline expired");
    return value;
  };
  const setPendingDeadline = (value: number) => {
    assert.ok(Number.isSafeInteger(value) && value > 0, "exact original deadline required");
    if (deadline !== undefined)
      assert.equal(value, deadline, "original deadline cannot be renewed");
    deadline = value;
    remaining();
  };
  const fetchImpl: typeof fetch = (url, init) => {
    assert.equal(init?.method, "GET", "integration qualification transport is read-only");
    const timeout = deadline === undefined ? 30_000 : Math.min(30_000, remaining());
    return (options.fetchImpl ?? fetch)(url, {
      ...init,
      signal: AbortSignal.any([
        ...(init?.signal ? [init.signal] : []),
        AbortSignal.timeout(timeout),
      ]),
    });
  };
  const sleep = async (milliseconds: number) => {
    await pause(deadline === undefined ? milliseconds : Math.min(milliseconds, remaining()));
    if (deadline !== undefined) remaining();
  };
  const common = {
    repository: options.repository,
    token: options.token,
    apiUrl: options.apiUrl ?? "https://api.github.com",
    fetchImpl,
    sleep,
  };
  const api: ReadApi = {
    read: async (resource) => JSON.parse((await githubApiRequest({ ...common, resource })).body),
    pages: (resource, collection, query = {}) =>
      githubApiPages({ ...common, resource, collection, query }),
  };
  return { api, setPendingDeadline };
}
