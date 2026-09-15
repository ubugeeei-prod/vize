import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  canTransition,
  createMemoryStatePersistence,
  createStateManifest,
  createStateStore,
  defineStateModel,
  defineStateTransitions,
  hydrateStateStore,
  persistStateSnapshot,
  restorePersistedState,
  serializeStateSnapshot,
  transitionState,
} from "./index.ts";

type CartState = {
  readonly items: readonly string[];
  readonly pending: boolean;
};

type CartAction =
  | { readonly type: "add"; readonly sku: string }
  | { readonly type: "checkout" }
  | { readonly type: "reset" };

const cart = defineStateModel({
  key: "cart.session",
  source: "../fixtures/CartPanel.vue",
  version: 2,
  initialState: { items: [], pending: false } as CartState,
  reducer(state, action: CartAction): CartState {
    switch (action.type) {
      case "add":
        return { ...state, items: [...state.items, action.sku] };
      case "checkout":
        return { ...state, pending: true };
      case "reset":
        return { items: [], pending: false };
    }
  },
  commands: {
    addMany(_context, skus: readonly string[]) {
      return skus.map((sku) => ({ type: "add", sku }) as const);
    },
  },
  persistence: {
    storageKey: "cart",
    ttlMs: 1_000,
    validate(state): state is CartState {
      return (
        typeof state === "object" &&
        state != null &&
        Array.isArray((state as CartState).items) &&
        typeof (state as CartState).pending === "boolean"
      );
    },
    migrate(state, context): CartState {
      assert.equal(context.fromVersion, 1);
      const legacy = state as { items?: readonly string[] };
      return { items: legacy.items ?? [], pending: false };
    },
  },
  meta: { owner: "commerce" },
} as const);

void test("dispatches reducers and commands with no request-global state", () => {
  const first = createStateStore(cart);
  const second = createStateStore(cart);

  assert.deepEqual(first.dispatch({ type: "add", sku: "book" }), {
    items: ["book"],
    pending: false,
  });
  assert.deepEqual(second.state, { items: [], pending: false });

  first.command("addMany", ["pen", "bag"]);
  assert.deepEqual(first.state.items, ["book", "pen", "bag"]);
});

void test("groups transactions and supports deterministic rollback", () => {
  const store = createStateStore(cart);
  const transaction = store.transaction([{ type: "add", sku: "book" }, { type: "checkout" }]);

  assert.deepEqual(transaction.before, { items: [], pending: false });
  assert.deepEqual(transaction.after, { items: ["book"], pending: true });

  store.rollback(transaction);
  assert.deepEqual(store.state, { items: [], pending: false });
});

void test("rolls back optimistic updates when confirmation fails", async () => {
  const store = createStateStore(cart);

  const committed = await store.optimistic({ type: "add", sku: "book" }, () => undefined);
  assert.equal(committed.status, "committed");
  assert.deepEqual(store.state.items, ["book"]);

  const rolledBack = await store.optimistic({ type: "add", sku: "bag" }, () => {
    throw new Error("payment failed");
  });
  assert.equal(rolledBack.status, "rolled-back");
  assert.deepEqual(store.state.items, ["book"]);
});

void test("serializes SSR state and hydrates identity per request", () => {
  const server = createStateStore(cart, { now: () => 500 });
  server.dispatch({ type: "add", sku: "book" });

  const snapshot = serializeStateSnapshot(server);
  assert.deepEqual(snapshot, {
    schemaVersion: 1,
    key: "cart.session",
    source: "../fixtures/CartPanel.vue",
    version: 2,
    state: { items: ["book"], pending: false },
    updatedAt: 500,
  });

  const client = hydrateStateStore(cart, snapshot, { now: () => 600 });
  const otherRequest = createStateStore(cart);
  client.dispatch({ type: "add", sku: "pen" });

  assert.deepEqual(client.state.items, ["book", "pen"]);
  assert.deepEqual(otherRequest.state.items, []);
});

void test("persists, migrates, expires, and recovers corrupted state", () => {
  const store = createStateStore(cart, { now: () => 100 });
  store.dispatch({ type: "add", sku: "book" });
  const adapter = createMemoryStatePersistence();

  persistStateSnapshot(store, adapter);
  assert.deepEqual(restorePersistedState(cart, adapter, { now: () => 200 }), {
    status: "accepted",
    state: { items: ["book"], pending: false },
    recovered: false,
  });

  adapter.setItem(
    "cart",
    JSON.stringify({
      schemaVersion: 1,
      key: "cart.session",
      source: "../fixtures/CartPanel.vue",
      version: 1,
      state: { items: ["legacy"] },
      updatedAt: 250,
    }),
  );
  assert.deepEqual(restorePersistedState(cart, adapter, { now: () => 300 }), {
    status: "migrated",
    state: { items: ["legacy"], pending: false },
    recovered: false,
  });

  adapter.setItem("cart", "{");
  assert.deepEqual(restorePersistedState(cart, adapter), {
    status: "invalid",
    state: { items: [], pending: false },
    recovered: true,
  });

  persistStateSnapshot(store, adapter);
  assert.deepEqual(restorePersistedState(cart, adapter, { now: () => 2_000 }), {
    status: "expired",
    state: { items: [], pending: false },
    recovered: true,
  });
});

void test("emits generator metadata from source-owned Vue fixtures", () => {
  assert.deepEqual(createStateManifest([cart]), {
    schemaVersion: 1,
    models: [
      {
        key: "cart.session",
        source: "../fixtures/CartPanel.vue",
        version: 2,
        meta: { owner: "commerce" },
        persistence: true,
      },
    ],
  });

  const fixture = readFileSync(new URL("../fixtures/CartPanel.vue", import.meta.url), "utf8");
  assert.match(fixture, /<template>/);
  assert.match(fixture, /<script setup lang="ts">/);
});

void test("finite transitions are validated at runtime", () => {
  const table = defineStateTransitions({
    idle: ["editing"],
    editing: ["saving", "idle"],
    saving: ["saved", "failed"],
    saved: ["editing"],
    failed: ["editing"],
  } as const);

  assert.equal(canTransition(table, "editing", "saving"), true);
  assert.equal(transitionState(table, "saving", "saved"), "saved");
  assert.throws(() => transitionState(table, "idle", "saved" as never), /Invalid state transition/);
});

void test("invalid model definitions fail closed", () => {
  assert.throws(
    () =>
      defineStateModel({
        key: "broken",
        // @ts-expect-error Runtime diagnostics still protect untyped callers.
        source: "./broken.ts",
        version: 1,
        initialState: {},
        reducer: (state) => state,
      }),
    /VIZE_STATE_SOURCE_NOT_VUE/,
  );
  assert.throws(
    () =>
      defineStateModel({
        key: "broken",
        source: "./Broken.vue",
        version: 0,
        initialState: {},
        reducer: (state) => state,
      }),
    /VIZE_STATE_INVALID_VERSION/,
  );
});
