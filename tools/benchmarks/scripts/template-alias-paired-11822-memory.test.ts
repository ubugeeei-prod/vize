import assert from "node:assert/strict";
import { test } from "node:test";
import { parseRollup, parseStat } from "./template-alias-paired-11822-memory.ts";

void test("Linux identities keep kernel start ticks across names containing parentheses", () => {
  const fields = ["S", "12", ...Array(17).fill("0"), "1234567890123456789", "0"];
  assert.deepEqual(parseStat("345 (mapper (worker)) " + fields.join(" ")), {
    pid: 345,
    ppid: 12,
    state: "S",
    startTicks: "1234567890123456789",
  });
  assert.throws(() => parseStat("345 mapper"));
  fields[19] = "not-an-identity";
  assert.throws(() => parseStat("345 (worker) " + fields.join(" ")));
});

void test("RSS and PSS remain separate and missing/duplicate fields fail", () => {
  assert.deepEqual(parseRollup("Rss: 1024 kB\nPss: 512 kB\nPrivate_Clean: 200 kB\n"), {
    rssKiB: 1024,
    pssKiB: 512,
  });
  assert.throws(() => parseRollup("Rss: 1024 kB\n"));
  assert.throws(() => parseRollup("Rss: 1024 kB\nPss: 512 kB\nPss: 0 kB\n"));
});
