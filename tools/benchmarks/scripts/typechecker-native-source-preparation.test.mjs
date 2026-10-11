import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  awaitReleasePreparation,
  selectSourceRecipe,
  sourcePreparationModule,
} from "./typechecker-native-source-recipe.mjs";
import { commit, fixture, git, put } from "./typechecker-native-source-recipe-fixture.mjs";

function incomplete(pr) {
  return {
    ...structuredClone(pr),
    body: pr.body
      .split("\n")
      .filter((line) => !line.startsWith("<!-- vize-release-integration:"))
      .join("\n"),
  };
}

function preparation(f, snapshots, options = {}) {
  let clock = 0,
    reads = 0,
    loads = 0;
  const opened = incomplete(f.event.pull_request);
  const state = {
    now: () => clock,
    pause: async (ms) => {
      clock += ms;
    },
    observe: async () => structuredClone(snapshots[Math.min(reads++, snapshots.length - 1)]),
    loadPin: async (pin) => {
      loads++;
      return pin;
    },
    timeoutMs: 6_000,
    ...options,
  };
  return {
    run: () => awaitReleasePreparation({ ...f, event: { pull_request: opened }, ...state }),
    counts: () => ({ clock, reads, loads }),
  };
}

const observation = (f, pullRequest = f.event.pull_request, pin = f.pin) => ({
  pullRequest: structuredClone(pullRequest),
  source: f.source,
  pin,
});

await test("runtime preparation uses whole workflow-object module bytes and refuses missing authority", async () => {
  await fixture(async (f) => {
    const path = "tools/benchmarks/scripts/typechecker-native-source-preparation.mjs";
    const bytes = readFileSync(
      new URL("./typechecker-native-source-preparation.mjs", import.meta.url),
    );
    put(f.root, path, bytes);
    const workflow = commit(f.root);
    git(f.root, "checkout", "--detach", f.source);
    put(f.root, path, 'throw new Error("checkout module must never execute");\n');
    const module = await sourcePreparationModule(f.root, workflow);
    const result = await awaitReleasePreparation(
      {
        ...f,
        env: { ...f.env, GITHUB_WORKFLOW_SHA: workflow },
        observe: async () => observation(f),
        loadPin: async (pin) => pin,
      },
      module,
    );
    const expected = selectSourceRecipe(f);
    assert.equal(result.recipe, expected.recipe);
    assert.deepEqual(result.originalEnvironment, expected.originalEnvironment);
    assert.deepEqual(result.changes, expected.changes);
    await assert.rejects(sourcePreparationModule(f.root, f.source), {
      code: "ERR_ASSERTION",
      actual: 128,
    });
    await assert.rejects(sourcePreparationModule(f.root, "main"), /AssertionError/u);
  });
});

await test("opened receipt waits for actual pin and complete live metadata, then selects the whole original source step", async () => {
  await fixture(async (f) => {
    const initial = incomplete(f.event.pull_request);
    const replay = preparation(f, [
      observation(f, initial, null),
      observation(f, initial),
      observation(f),
    ]);
    const result = await replay.run();
    const expected = selectSourceRecipe(f);
    assert.equal(result.mode, "immutable-source-step");
    assert.equal(result.recipe, expected.recipe);
    assert.equal(result.recipeSha256, expected.recipeSha256);
    assert.deepEqual(result.changes, expected.changes);
    assert.deepEqual(result.originalEnvironment, expected.originalEnvironment);
    assert.equal(result.pin, f.pin);
    assert.deepEqual(result.authentication.preparation.initialPullRequest, initial);
    assert.deepEqual(result.authentication.preparation.observations, [
      { elapsedMs: 0, pin: null, integration: null },
      { elapsedMs: 3_000, pin: f.pin, integration: null },
      { elapsedMs: 6_000, pin: f.pin, integration: 8250 },
    ]);
    assert.deepEqual(replay.counts(), { clock: 6_000, reads: 3, loads: 1 });
  });
});

await test("a complete receipt executes immediately and never accepts missing pin custody", async () => {
  await fixture(async (f) => {
    const ready = preparation(f, [observation(f)], { event: f.event });
    assert.equal((await ready.run()).recipe, selectSourceRecipe(f).recipe);
    assert.deepEqual(ready.counts(), { clock: 0, reads: 1, loads: 1 });
    const missing = preparation(f, [observation(f, f.event.pull_request, null)], {
      event: f.event,
    });
    await assert.rejects(missing.run(), /Complete release receipt has no immutable pin/u);
    assert.deepEqual(missing.counts(), { clock: 0, reads: 1, loads: 0 });
  });
});

await test("a permanently incomplete preparation fails within its finite deadline without selecting or loading a pin", async () => {
  await fixture(async (f) => {
    const missing = preparation(f, [observation(f, incomplete(f.event.pull_request), null)]);
    await assert.rejects(missing.run(), /preparation timed out/u);
    assert.deepEqual(missing.counts(), { clock: 6_000, reads: 3, loads: 0 });
    const late = preparation(f, [observation(f)], {
      now: (() => {
        let n = 0;
        return () => n++ * 7_000;
      })(),
    });
    await assert.rejects(late.run(), /preparation timed out/u);
    assert.equal(late.counts().reads, 0);
  });
});

await test("foreign, stale, closed, changed-author and altered live receipts fail before native recipe selection", async () => {
  for (const integrationCollision of [false, true]) {
    await fixture(async (f) => {
      if (integrationCollision)
        f.event.pull_request.body =
          "Harmless incident reference: 28056b4f3bca41f2e0ca825017e87605c5c965a3\n" +
          f.event.pull_request.body;
      assert.equal(
        git(f.root, "rev-list", "--parents", "-n", "1", f.source),
        `${f.source} ${f.cut}`,
      );
      const controls = [
        (row) => {
          row.pullRequest.head.repo.full_name = "foreign/repository";
        },
        (row) => {
          row.pullRequest.base.repo.full_name = "foreign/repository";
        },
        (row) => {
          row.pullRequest.head.sha = f.cut;
        },
        (row) => {
          row.source = f.cut;
        },
        (row) => {
          row.source = null;
        },
        (row) => {
          row.pullRequest.number++;
        },
        (row) => {
          row.pullRequest.user.login = "other-maintainer";
        },
        (row) => {
          row.pullRequest.state = "closed";
        },
        (row) => {
          row.pullRequest.merged = true;
        },
        (row) => {
          row.pullRequest.draft = false;
        },
        (row) => {
          row.pullRequest.body = row.pullRequest.body.replace(f.cut, f.source);
        },
        (row) => {
          row.pullRequest.body += "\n<!-- vize-release-pin: immutable-v1 -->";
        },
      ];
      for (const alter of controls) {
        const row = observation(f);
        alter(row);
        const forged = preparation(f, [row]);
        await assert.rejects(forged.run());
        assert.equal(forged.counts().loads, 0);
      }
      const changedIntegration = observation(f);
      changedIntegration.pullRequest.body = changedIntegration.pullRequest.body.replace(
        "<!-- vize-release-integration: 8250 -->",
        "<!-- vize-release-integration: 8251 -->",
      );
      assert.equal(
        changedIntegration.pullRequest.body.replace(
          "<!-- vize-release-integration: 8251 -->",
          "<!-- vize-release-integration: 8250 -->",
        ),
        f.event.pull_request.body,
      );
      const changed = preparation(f, [changedIntegration], { event: f.event });
      await assert.rejects(changed.run(), /Complete integration receipt changed/u);
      assert.equal(changed.counts().loads, 0);
      const lost = preparation(f, [observation(f, incomplete(f.event.pull_request))], {
        event: f.event,
      });
      await assert.rejects(lost.run(), /Complete integration receipt changed/u);
    });
  }
});

await test("invalid original context and unprivileged author never enter the preparation polling loop", async () => {
  await fixture(async (f) => {
    for (const options of [
      { permission: { role_name: "write" } },
      { env: { ...f.env, GITHUB_REPOSITORY: "foreign/repository" } },
      { env: { ...f.env, MAIN_SOURCE_SHA: f.source } },
      { env: { ...f.env, GITHUB_EVENT_NAME: "workflow_dispatch" } },
    ]) {
      const invalid = preparation(f, [observation(f)], options);
      await assert.rejects(invalid.run());
      assert.deepEqual(invalid.counts(), { clock: 0, reads: 0, loads: 0 });
    }
  });
});

await test("wrong pin parent, tree, receipt message and fetch identity never fall back to current native commands", async () => {
  await fixture(async (f) => {
    const pins = [
      f.source,
      git(
        f.root,
        "commit-tree",
        git(f.root, "rev-parse", `${f.cut}^{tree}`),
        "-p",
        f.source,
        "-m",
        "foreign tree",
      ),
      git(
        f.root,
        "commit-tree",
        git(f.root, "rev-parse", `${f.source}^{tree}`),
        "-p",
        f.source,
        "-m",
        "foreign receipt",
      ),
    ];
    for (const pin of pins)
      await assert.rejects(preparation(f, [observation(f, f.event.pull_request, pin)]).run());
    await assert.rejects(
      preparation(f, [observation(f)], { loadPin: async () => f.source }).run(),
      /Fetched immutable pin changed/u,
    );
  });
});

await test("a pin advertised during preparation cannot disappear or change before final receipt selection", async () => {
  await fixture(async (f) => {
    const first = observation(f, incomplete(f.event.pull_request));
    for (const pin of [null, f.source])
      await assert.rejects(
        preparation(f, [first, observation(f, f.event.pull_request, pin)]).run(),
        /Advertised immutable pin changed during preparation/u,
      );
    let clock = 0;
    await assert.rejects(
      preparation(f, [observation(f)], {
        now: () => clock,
        loadPin: async (pin) => {
          clock = 6_001;
          return pin;
        },
      }).run(),
      /preparation timed out during pin fetch/u,
    );
  });
});

await test("whole metadata and real YAML recipe authentication must finish within the same preparation deadline", async () => {
  await fixture(async (f) => {
    let clock = 0;
    const slow = preparation(f, [observation(f)], {
      now: () => clock,
      parse: (bytes) => {
        const value = f.parse(bytes);
        clock = 6_001;
        return value;
      },
    });
    await assert.rejects(slow.run(), /preparation timed out during source recipe authentication/u);
    assert.equal(slow.counts().loads, 1);
  });
});
