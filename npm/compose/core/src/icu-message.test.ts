import assert from "node:assert/strict";
import { test } from "node:test";

import { createMessageFormatter, formatMessage, parseMessage } from "./icu-message.ts";
import type { MessageSyntaxError } from "./icu-message.ts";

void test("formats simple arguments and plural with #", () => {
  const message = "Hello {name}, you have {count, plural, one {# item} other {# items}}";
  assert.equal(
    formatMessage("en", message, { name: "Ada", count: 1 }),
    "Hello Ada, you have 1 item",
  );
  assert.equal(
    formatMessage("en", message, { name: "Ada", count: 1234 }),
    "Hello Ada, you have 1,234 items",
  );
});

void test("plural supports exact matches, offset, and locale rules", () => {
  const message =
    "{n, plural, offset:1 =0 {nobody} =1 {{who} alone} one {{who} and # other} other {{who} and # others}}";
  assert.equal(formatMessage("en", message, { n: 0, who: "Kim" }), "nobody");
  assert.equal(formatMessage("en", message, { n: 1, who: "Kim" }), "Kim alone");
  assert.equal(formatMessage("en", message, { n: 2, who: "Kim" }), "Kim and 1 other");
  assert.equal(formatMessage("en", message, { n: 3, who: "Kim" }), "Kim and 2 others");
  const polish = "{n, plural, one {# plik} few {# pliki} many {# plików} other {# pliku}}";
  assert.equal(formatMessage("pl", polish, { n: 3 }), "3 pliki");
  assert.equal(formatMessage("pl", polish, { n: 5 }), "5 plików");
});

void test("selectordinal and select", () => {
  const ordinal = "{place, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}";
  assert.deepEqual(
    [1, 2, 3, 4, 11, 22, 23].map((place) => formatMessage("en", ordinal, { place })),
    ["1st", "2nd", "3rd", "4th", "11th", "22nd", "23rd"],
  );
  const select = "{g, select, male {He} female {She} other {They}} left";
  assert.equal(formatMessage("en", select, { g: "female" }), "She left");
  assert.equal(formatMessage("en", select, { g: "other" }), "They left");
  assert.equal(
    createMessageFormatter("en").format(select, { g: "unknown" }),
    "They left",
    "unknown select values use other",
  );
});

void test("nested arguments inside cases", () => {
  const message =
    "{host, select, me {{guests, plural, =0 {You have no guests} other {You invited # guests including {first}}}} other {{host} hosts}}";
  // `host` is typed as the select keys; an unknown host needs the untyped formatter.
  assert.equal(
    createMessageFormatter("en").format(message, { host: "Bob", guests: 0, first: "" }),
    "Bob hosts",
  );
});

void test("number, date, and time styles are deterministic", () => {
  const message =
    "{a, number} {b, number, integer} {c, number, percent} {d, number, ::currency/EUR} {e, number, ::compact-short} {f, number, ::.00}";
  assert.equal(
    formatMessage("en", message, { a: 1234.5, b: 2.7, c: 0.25, d: 12.5, e: 12_345, f: 3 }),
    "1,234.5 3 25% €12.50 12K 3.00",
  );
  const when = "{d, date, short} {d, date} {d, date, full} {d, time, short}";
  assert.equal(
    formatMessage("en", when, { d: Date.UTC(2026, 9, 24, 13, 5) }),
    "10/24/26 Oct 24, 2026 Saturday, October 24, 2026 1:05 PM",
  );
  assert.equal(
    formatMessage("en", "{d, time, short}", { d: 0 }, { timeZone: "Asia/Tokyo" }),
    "9:00 AM",
  );
  assert.equal(formatMessage("ja", "{d, date, long}", { d: new Date(0) }), "1970年1月1日");
});

void test("named formats override built-in styles", () => {
  const formatter = createMessageFormatter("en", {
    numberFormats: { usd: { style: "currency", currency: "USD" } },
    dateTimeFormats: { monthOnly: { month: "long" } },
  });
  assert.equal(formatter.format("{p, number, usd}", { p: 5 }), "$5.00");
  assert.equal(formatter.format("{d, date, monthOnly}", { d: 0 }), "January");
});

void test("apostrophe quoting", () => {
  assert.equal(formatMessage("en", "It''s {x}", { x: "ok" }), "It's ok");
  assert.equal(formatMessage("en", "'{literal}' {x}", { x: "y" }), "{literal} y");
  assert.equal(formatMessage("en", "don't", {}), "don't");
  assert.equal(formatMessage("en", "{n, plural, other {'#' is # }}", { n: 2 }), "# is 2 ");
});

void test("missing simple arguments render their placeholder", () => {
  assert.equal(createMessageFormatter("en").format("Hi {name}", {}), "Hi {name}");
});

void test("parseMessage builds an AST and rejects malformed input", () => {
  assert.deepEqual(parseMessage("a {b} c"), [
    { kind: "text", value: "a " },
    { kind: "argument", name: "b" },
    { kind: "text", value: " c" },
  ]);
  for (const bad of [
    "{",
    "{a",
    "{a, plural, one {x}}",
    "}",
    "{a, unknown}",
    "{a, select, x {y} other }",
  ]) {
    assert.throws(
      () => parseMessage(bad),
      (error: unknown) =>
        error instanceof SyntaxError &&
        (error as MessageSyntaxError).code === "VIZE_COMPOSE_ICU_SYNTAX",
      bad,
    );
  }
});

void test("formatters cache parsed messages per instance", () => {
  const formatter = createMessageFormatter("en");
  const nodes = parseMessage("{n, number}");
  assert.equal(formatter.format(nodes, { n: 1000 }), "1,000");
  assert.equal(formatter.format("{n, number}", { n: 2000 }), "2,000");
  assert.equal(formatter.locale, "en");
});
