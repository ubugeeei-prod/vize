import assert from "node:assert/strict";

/** Only the original incomplete receipt may wait; completed, changed or foreign cuts fail closed. */
export async function awaitReleasePreparation({
  root,
  env,
  event,
  permission,
  parse,
  observe,
  loadPin,
  now = () => performance.now(),
  pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  timeoutMs = 300_000,
  sourceContext,
  releaseIdentity,
  sha,
  selectSourceRecipe,
}) {
  sourceContext(root, env, event);
  const initial = releaseIdentity(root, env, event.pull_request, permission, true);
  const started = now(),
    observations = [];
  const deadline = (stage = "") =>
    assert.ok(
      now() - started <= timeoutMs,
      `Immutable release preparation timed out${stage}; no source recipe selected`,
    );
  let advertisedPin;
  while (true) {
    deadline();
    const observed = await observe();
    const live = releaseIdentity(root, env, observed.pullRequest, permission, true);
    for (const key of ["source", "cut", "old", "next", "number", "author"])
      assert.equal(live[key], initial[key], `Immutable release ${key} changed during preparation`);
    if (initial.integration !== null)
      assert.equal(live.integration, initial.integration, "Complete integration receipt changed");
    assert.equal(observed.source, initial.source, "Immutable source branch changed");
    if (observed.pin !== null) sha(observed.pin);
    if (advertisedPin !== undefined)
      assert.equal(
        observed.pin,
        advertisedPin,
        "Advertised immutable pin changed during preparation",
      );
    if (observed.pin !== null) advertisedPin = observed.pin;
    observations.push({
      elapsedMs: now() - started,
      pin: observed.pin,
      integration: live.integration,
    });
    if (live.integration !== null) {
      assert.ok(observed.pin !== null, "Complete release receipt has no immutable pin");
      deadline();
      const pin = await loadPin(observed.pin);
      assert.equal(pin, observed.pin, "Fetched immutable pin changed");
      deadline(" during pin fetch");
      const selected = selectSourceRecipe({
        root,
        env,
        event: { ...event, pull_request: observed.pullRequest },
        permission,
        pin,
        parse,
      });
      deadline(" during source recipe authentication");
      selected.authentication.preparation = {
        initialPullRequest: event.pull_request,
        observations,
      };
      return selected;
    }
    assert.ok(initial.integration === null, "Complete receipt cannot return to preparation");
    assert.ok(
      now() - started < timeoutMs,
      "Immutable release preparation timed out; no source recipe selected",
    );
    await pause(Math.min(3_000, timeoutMs - (now() - started)));
  }
}
