import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { renderComposableOnServer, withTrappedServerGlobals } from "./testing/ssr-harness.ts";
import {
  COOKIE_ADAPTER_KEY,
  parseCookieHeader,
  provideCookieAdapter,
  serializeCookie,
  useCookie,
} from "./use-cookie.ts";
import type { CookieAdapter, CookieFailure } from "./use-cookie.ts";

class MemoryAdapter implements CookieAdapter {
  header: string;
  readonly written: string[] = [];

  constructor(header = "") {
    this.header = header;
  }

  readonly read = (): string => this.header;

  readonly write = (setCookie: string): void => {
    this.written.push(setCookie);
  };
}

void test("parses cookie headers with decoding, quotes, and first-wins", () => {
  assert.deepEqual(parseCookieHeader('a=1; b=hello%20world; c="q"; a=2; bad; =x; d=%E0'), {
    a: "1",
    b: "hello world",
    c: "q",
    d: "%E0",
  });
  assert.deepEqual(parseCookieHeader(""), {});
});

void test("serializes cookies with every attribute", () => {
  assert.equal(
    serializeCookie("session", "a b;c", {
      path: "/app",
      domain: "example.com",
      maxAge: 60,
      expires: new Date(0),
      sameSite: "none",
      secure: true,
      httpOnly: true,
      partitioned: true,
    }),
    "session=a%20b%3Bc; Max-Age=60; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Domain=example.com; Path=/app; SameSite=None; Secure; HttpOnly; Partitioned",
  );
  assert.equal(serializeCookie("x", "1", { sameSite: "lax" }), "x=1; SameSite=Lax");
  assert.equal(serializeCookie("x", "1", { sameSite: "strict" }), "x=1; SameSite=Strict");
});

void test("rejects malformed names and attributes with tagged errors", () => {
  assert.throws(() => serializeCookie("a b", "1"), /VIZE_COMPOSE_COOKIE_INVALID_NAME/);
  assert.throws(() => serializeCookie("a", "1", { maxAge: 1.5 }), /INVALID_MAX_AGE/);
  assert.throws(() => serializeCookie("a", "1", { expires: new Date(Number.NaN) }), /EXPIRES/);
  assert.throws(() => serializeCookie("a", "1", { domain: "exa mple" }), /INVALID_DOMAIN/);
  assert.throws(() => serializeCookie("a", "1", { path: "/;x" }), /INVALID_PATH/);
  assert.throws(() => serializeCookie("a", "1", { sameSite: "none" }), /REQUIRES_SECURE/);
  assert.throws(() => serializeCookie("a", "1", { partitioned: true }), /REQUIRES_SECURE/);
});

void test("reads a raw string cookie and writes assignments", () => {
  const adapter = new MemoryAdapter("theme=dark");
  const cookie = useCookie("theme", { adapter, maxAge: 10 });

  assert.equal(cookie.supported.value, true);
  assert.equal(cookie.state.value, "dark");
  cookie.state.value = "light";
  assert.deepEqual(adapter.written, ["theme=light; Max-Age=10; Path=/"]);
  cookie.state.value = undefined;
  assert.equal(
    adapter.written.at(-1),
    "theme=; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/",
  );
});

void test("infers JSON for non-string defaults and validates the kind", () => {
  const failures: CookieFailure[] = [];
  const adapter = new MemoryAdapter(`prefs=${encodeURIComponent('{"dense":true}')}; n=abc`);
  const prefs = useCookie("prefs", { adapter, default: { dense: false } });
  const count = useCookie("n", {
    adapter,
    default: 0,
    onError: (failure) => failures.push(failure),
  });

  assert.deepEqual(prefs.state.value, { dense: true });
  assert.equal(count.state.value, 0);
  assert.equal(count.error.value?.code, "read-failed");
  assert.equal(failures.length, 1);
  assert.equal(adapter.written.length, 0, "reading never writes");

  prefs.state.value = { dense: false };
  assert.equal(adapter.written.at(-1), `prefs=${encodeURIComponent('{"dense":false}')}; Path=/`);
});

void test("runs the validation hook and falls back on rejection", () => {
  type Locale = "en" | "ja";
  const adapter = new MemoryAdapter("locale=fr");
  const locale = useCookie<Locale>("locale", {
    adapter,
    default: "en",
    validate: (candidate): candidate is Locale => candidate === "en" || candidate === "ja",
  });
  assert.equal(locale.state.value, "en");
  assert.equal(locale.error.value?.code, "invalid-value");

  adapter.header = "locale=ja";
  locale.refresh();
  assert.equal(locale.state.value, "ja");
  assert.equal(locale.error.value, undefined);
  assert.equal(adapter.written.length, 0);
});

void test("remove expires the cookie and restores the default without rewriting it", () => {
  const adapter = new MemoryAdapter("n=5");
  const count = useCookie("n", { adapter, default: 1 });
  count.remove();
  assert.equal(count.state.value, 1);
  assert.deepEqual(adapter.written, [
    "n=; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/",
  ]);
});

void test("reports write failures without throwing", () => {
  const cookie = useCookie("x", {
    adapter: {
      read: () => "",
      write: () => {
        throw new Error("headers already sent");
      },
    },
  });
  cookie.state.value = "1";
  assert.equal(cookie.error.value?.code, "write-failed");
  const invalid = useCookie("bad name", { adapter: new MemoryAdapter() });
  invalid.state.value = "1";
  assert.equal(invalid.error.value?.code, "write-failed");
});

void test("stays on the default without an adapter", () => {
  const cookie = useCookie("x", { adapter: null, default: 3 });
  assert.equal(cookie.supported.value, false);
  cookie.state.value = 4;
  assert.equal(cookie.error.value, undefined);
});

void test("server rendering without an adapter uses the default", async () => {
  const state = await renderComposableOnServer(() => {
    const cookie = useCookie("theme", { default: "light" });
    return { theme: cookie.state, supported: cookie.supported };
  });
  assert.equal(state, '{"theme":"light","supported":false}');
});

void test("server rendering reads the request through an explicit adapter", async () => {
  const adapter = new MemoryAdapter("theme=dark");
  const state = await renderComposableOnServer(() => {
    const cookie = useCookie("theme", { default: "light", adapter });
    return { theme: cookie.state };
  });
  assert.equal(state, '{"theme":"dark"}');
});

void test("server rendering uses an app-provided adapter for request and response", async () => {
  const adapter = new MemoryAdapter(`cart=${encodeURIComponent("[1,2]")}`);
  const Child = defineComponent({
    setup() {
      const cart = useCookie("cart", { default: [] as number[] });
      const visits = useCookie("visits", { default: 0, httpOnly: true });
      visits.state.value += 1;
      return () => h("p", `${cart.state.value.join(",")}|${String(visits.state.value)}`);
    },
  });
  const html = await withTrappedServerGlobals(async () => {
    const app = createSSRApp(Child);
    app.provide(COOKIE_ADAPTER_KEY, adapter);
    return renderToString(app);
  });
  assert.equal(html, "<p>1,2|1</p>");
  assert.deepEqual(adapter.written, ["visits=1; Path=/; HttpOnly"]);
});

void test("a parent component can provide the adapter to its subtree", async () => {
  const adapter = new MemoryAdapter("id=7");
  const Child = defineComponent({
    setup() {
      const id = useCookie("id");
      return () => h("i", id.state.value ?? "none");
    },
  });
  const Parent = defineComponent({
    setup() {
      provideCookieAdapter(adapter);
      return () => h(Child);
    },
  });
  const html = await withTrappedServerGlobals(() => renderToString(createSSRApp(Parent)));
  assert.equal(html, "<i>7</i>");
});
