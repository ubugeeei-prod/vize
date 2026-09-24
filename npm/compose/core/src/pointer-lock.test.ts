import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { PointerLockError, usePointerLock } from "./pointer-lock.ts";
import { asDocument, asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("locks and unlocks through confirmation events", async () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const scope = effectScope();
  const lock = scope.run(() => usePointerLock(asElement(element), { host: asDocument(document) }));
  assert.ok(lock);
  assert.equal(lock.isSupported.value, true);

  assert.equal(await lock.lock(), asElement(element));
  assert.equal(lock.element.value, asElement(element));
  assert.equal(lock.isLocked.value, true);

  assert.equal(await lock.unlock(), true);
  assert.equal(lock.element.value, null);
  assert.equal(lock.isLocked.value, false);
  assert.equal(await lock.unlock(), false);
  scope.stop();
});

void test("rejects with stable codes", async () => {
  const document = new FakeDocument();
  const refused = document.createElement();
  refused.refusePointerLock = true;
  const lock = usePointerLock(undefined, { host: asDocument(document) });

  await assert.rejects(lock.lock(), (error: unknown) => {
    assert.ok(error instanceof PointerLockError);
    assert.equal(error.code, "VIZE_COMPOSE_POINTER_LOCK_NO_TARGET");
    return true;
  });
  await assert.rejects(lock.lock(asElement(refused)), { code: "VIZE_COMPOSE_POINTER_LOCK_FAILED" });

  const server = usePointerLock(asElement(refused), { host: () => undefined });
  assert.equal(server.isSupported.value, false);
  await assert.rejects(server.lock(), { code: "VIZE_COMPOSE_POINTER_LOCK_UNSUPPORTED" });
});
