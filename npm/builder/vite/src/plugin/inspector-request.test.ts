import assert from "node:assert/strict";
import type { IncomingMessage } from "node:http";
import { test } from "node:test";

import { isCrossSiteRequest, isSameOriginRequest } from "./inspector-request.ts";

const request = (headers: IncomingMessage["headers"]) => ({ headers });
const host = "localhost:5173";

void test("a request from another site names itself in Sec-Fetch-Site or in its origin", () => {
  for (const headers of [
    { host, "sec-fetch-site": "cross-site" },
    { host, "sec-fetch-site": "cross-site", origin: `http://${host}` },
    { host, origin: "https://other.example" },
    { host, origin: "http://localhost:3000" },
    { host, origin: "https://localhost:5173.other.example" },
    { host, origin: "null" },
    { origin: `http://${host}` },
  ]) {
    assert.equal(isCrossSiteRequest(request(headers)), true, JSON.stringify(headers));
  }
  for (const headers of [
    {},
    { host },
    { host, "sec-fetch-site": "same-origin" },
    { host, "sec-fetch-site": "same-site" },
    { host, "sec-fetch-site": "none" },
    { host, origin: `http://${host}` },
    { host, origin: `https://${host}` },
    { ":authority": host, origin: `https://${host}` },
    { host: "app.example", origin: "https://app.example" },
  ]) {
    assert.equal(isCrossSiteRequest(request(headers)), false, JSON.stringify(headers));
  }
});

void test("a same-origin request says so, or names the host it asked as its origin", () => {
  for (const headers of [
    { host, "sec-fetch-site": "same-origin" },
    { host, "sec-fetch-site": "none" },
    { host, origin: `http://${host}` },
    { ":authority": host, origin: `https://${host}` },
  ]) {
    assert.equal(isSameOriginRequest(request(headers)), true, JSON.stringify(headers));
  }
  for (const headers of [
    {},
    { host },
    { host, "sec-fetch-site": "same-site" },
    { host, "sec-fetch-site": "cross-site" },
    { host, "sec-fetch-site": "same-site", origin: `http://${host}` },
    { host, origin: "http://localhost:3000" },
    { host, origin: "null" },
  ]) {
    assert.equal(isSameOriginRequest(request(headers)), false, JSON.stringify(headers));
  }
});
