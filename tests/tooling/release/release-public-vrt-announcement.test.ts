import assert from "node:assert/strict";
import { test } from "node:test";
import { VrtSecrets } from "../../../tools/support/release/public_acceptance/vrt_secrets.ts";
import {
  PublishedVrtAnnouncement,
  publishedTokenNotice,
} from "../../../tools/support/release/public_acceptance/vrt_service_announcement.ts";

const control = Buffer.alloc(32, 2).toString("base64url");
function prepared() {
  const secrets = new VrtSecrets();
  const parsed = new PublishedVrtAnnouncement(secrets);
  return { secrets, parsed };
}
function endpoint(parsed: PublishedVrtAnnouncement, value = "http://127.0.0.1:40031") {
  assert.equal(parsed.line(`  ${publishedTokenNotice}`), false);
  return parsed.line(`  VRT endpoint: ${value}`);
}

// Exact protocol/input controls only: no installed CLI, server, browser or tunnel is launched.
test("the authored published notice/endpoint/token order yields one volatile session", () => {
  const { secrets, parsed } = prepared();
  try {
    assert.equal(parsed.line(""), false);
    assert.equal(endpoint(parsed), false);
    assert.equal(parsed.line(`  Session token: ${control}`), true);
    const session = parsed.take();
    assert.equal(session.endpoint, "http://127.0.0.1:40031");
    assert.equal(session.token, control);
    assert.equal(session.tokenPrivacyNotice, true);
    assert.throws(() => secrets.assertSafe(control), /Credential-bearing retained data refused/);
    assert.throws(() => parsed.take(), /already taken/);
  } finally {
    parsed.forget();
    secrets.erase();
  }
});

test("missing notice/endpoint/credential and duplicate protocol fields refuse", () => {
  const cases = [
    (p: PublishedVrtAnnouncement) => p.line("VRT endpoint: http://127.0.0.1:40031"),
    (p: PublishedVrtAnnouncement) => p.line(`Session token: ${control}`),
    (p: PublishedVrtAnnouncement) => {
      p.line(publishedTokenNotice);
      p.line(publishedTokenNotice);
    },
    (p: PublishedVrtAnnouncement) => {
      endpoint(p);
      p.line("VRT endpoint: http://127.0.0.1:40031");
    },
    (p: PublishedVrtAnnouncement) => {
      endpoint(p);
      p.line(`Session token: ${control}`);
      p.line(`Session token: ${control}`);
    },
    (p: PublishedVrtAnnouncement) => {
      endpoint(p);
      p.take();
    },
  ];
  for (const invoke of cases) {
    const { secrets, parsed } = prepared();
    try {
      assert.throws(() => invoke(parsed));
    } finally {
      parsed.forget();
      secrets.erase();
    }
  }
});

test("non-loopback, noncanonical, credential-bearing and path/query endpoints refuse", () => {
  for (const value of [
    "http://localhost:40031",
    "http://127.0.0.1",
    "https://127.0.0.1:40031",
    "http://user:control@127.0.0.1:40031",
    "http://127.0.0.1:40031/",
    "http://127.0.0.1:40031?token=control",
    "invalid",
  ]) {
    const { secrets, parsed } = prepared();
    try {
      assert.throws(() => endpoint(parsed, value), /endpoint refused/);
    } finally {
      parsed.forget();
      secrets.erase();
    }
  }
});

test("invalid token and oversized row diagnostics contain no supplied row bytes", () => {
  for (const value of ["!".repeat(43), "control-malformed-secret"]) {
    const { secrets, parsed } = prepared();
    try {
      endpoint(parsed);
      assert.throws(
        () => parsed.line(`Session token: ${value}`),
        (error) => {
          assert.ok(error instanceof Error);
          assert.equal(error.message.includes(value), false);
          return true;
        },
      );
    } finally {
      parsed.forget();
      secrets.erase();
    }
  }
  const { secrets, parsed } = prepared();
  try {
    assert.throws(() => parsed.line(control.repeat(400)), /row too large/);
  } finally {
    parsed.forget();
    secrets.erase();
  }
});

test("forget prevents reopening or reusing the parsed credential before global scope erasure", () => {
  const { secrets, parsed } = prepared();
  try {
    endpoint(parsed);
    parsed.line(`Session token: ${control}`);
    parsed.forget();
    assert.throws(() => parsed.take(), /incomplete/);
    assert.throws(() => parsed.line("VRT endpoint: http://127.0.0.1:40031"), /Closed published/);
    assert.throws(() => parsed.line(`Session token: ${control}`), /Closed published/);
    assert.throws(() => secrets.assertSafe(control), /Credential-bearing retained data refused/);
  } finally {
    secrets.erase();
  }
});
