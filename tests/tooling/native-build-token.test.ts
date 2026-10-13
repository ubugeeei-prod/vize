// The native image token laws for `tools/support/github/build-napi-package.ts`.
// Native Smoke's selected producer runs exactly this file before it builds.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  nativeBuildEnvironment,
  nativeBuildToken,
  platformEntropy,
} from "../../tools/support/github/build-napi-package.ts";

function assertNativeBuildToken(token: string | undefined): asserts token is string {
  assert.equal(token?.length, 32);
  for (const character of token) {
    assert.ok(("0" <= character && character <= "9") || ("a" <= character && character <= "f"));
  }
}

void test("native build token preserves every byte including zero-padded nibbles", () => {
  const bytes = Uint8Array.from([
    0x00, 0x01, 0x0f, 0x10, 0xff, 0x00, 0x01, 0x0f, 0x10, 0xff, 0x00, 0x01, 0x0f, 0x10, 0xff, 0x00,
  ]);
  const token = nativeBuildToken(bytes);
  assert.equal(token, "00010f10ff00010f10ff00010f10ff00");
  assertNativeBuildToken(token);
});

void test("actual platform entropy yields fresh 128-bit native build tokens", () => {
  const first = platformEntropy(16);
  const second = platformEntropy(16);
  assert.ok(first, "Native build producer requires platform entropy");
  assert.ok(second, "Native build producer requires platform entropy");
  assert.equal(first.length, 16);
  assert.equal(second.length, 16);
  const firstToken = nativeBuildToken(first);
  const secondToken = nativeBuildToken(second);
  assertNativeBuildToken(firstToken);
  assertNativeBuildToken(secondToken);
  assert.notEqual(firstToken, secondToken);
});

void test("production build environment preserves inheritance and overrides old tokens", () => {
  const tokenKey = "VIZE_NATIVE_BUILD_TOKEN";
  const probeKey = "VIZE_NATIVE_BUILD_TOKEN_WBTEST_PROBE";
  const emptyKey = "VIZE_NATIVE_BUILD_TOKEN_WBTEST_EMPTY";
  const originals = [tokenKey, probeKey, emptyKey].map((key) => [key, process.env[key]] as const);
  const inheritedToken = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  try {
    process.env[tokenKey] = inheritedToken;
    process.env[probeKey] = "native build inheritance 日本語\nsecond line";
    process.env[emptyKey] = "";
    const inherited = { ...process.env };
    const first = nativeBuildEnvironment();
    const second = nativeBuildEnvironment();
    assert.ok(first, "Production native build environment requires entropy");
    assert.ok(second, "Production native build environment requires entropy");
    assert.equal(Object.keys(first).length, Object.keys(inherited).length);
    assert.equal(Object.keys(second).length, Object.keys(inherited).length);
    for (const [key, value] of Object.entries(inherited)) {
      if (key !== tokenKey) {
        // Do not print inherited environment values in test failure output.
        assert.ok(first[key] === value);
        assert.ok(second[key] === value);
      }
    }
    const firstToken = first[tokenKey];
    const secondToken = second[tokenKey];
    assertNativeBuildToken(firstToken);
    assertNativeBuildToken(secondToken);
    assert.notEqual(firstToken, inheritedToken);
    assert.notEqual(secondToken, inheritedToken);
    assert.notEqual(firstToken, secondToken);
    assert.equal(process.env[tokenKey], inheritedToken);
  } finally {
    for (const [key, value] of originals) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
});
