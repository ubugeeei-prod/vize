import assert from "node:assert/strict";
import { test } from "node:test";
import { verifyBetaLogicalRoute } from "./support/native-beta-dto/logical-attached-route.ts";
import {
  sdk4CleanupPackages,
  verifyAfter4CleanupSource,
} from "./support/native-beta-dto/after4-cleanup-source.ts";
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
import {
  sdk4CleanupFixture as fixture,
  observer,
  helper,
  input,
  registry,
} from "./support/native-beta-sdk4-cleanup-fixture.ts";
const uri = (row: unknown) => object(object(object(row).params).textDocument).uri;
type Fixture = ReturnType<typeof fixture>;
const authority = (f: Fixture) =>
  verifyAfter4CleanupSource(
    f.sourceRecord,
    (file) => {
      const bytes = f.files.get(file);
      assert.ok(bytes);
      return bytes;
    },
    Buffer.from(f.artifacts.map((row) => JSON.stringify(row)).join("\n") + "\n"),
  );
const verify = (f: Fixture, bound = authority(f)) =>
  verifyBetaLogicalRoute(f.owners, f.context, f.source, f.root, f.phases, f.attached, bound);
const changesRefuse = (changes: Array<(f: Fixture) => void>) => {
  for (const change of changes) {
    const f = fixture();
    change(f);
    assert.throws(() => verify(f));
  }
};

test("AFTER4 source/compiler/full55/API26 precede exact source cleanup; default stays strict", () => {
  const f = fixture();
  assert.throws(() => verify(f, null));
  const proof = verify(f),
    row = object(object(proof.owners[0]!.sourceSupplement).sourceCleanup);
  assert.equal(proof.owners[0]!.semanticPackets.length, 55);
  assert.equal(proof.owners[0]!.complete, true);
  assert.equal((row.packets as unknown[]).length, 5);
  assert.equal((row.apiPairs as unknown[]).length, 26);
  assert.equal(row.apiClient, f.owners[0]!.apiClient);
  assert.equal(row.apiServer, f.owners[0]!.apiServer);
  assert.equal(object(row.sourceCompilerAuthority).nativeQualification, false);
});
test("the actual asynchronous capability error/release order remains whole and unnormalized", () => {
  const f = fixture(),
    rows = f.owners[0]!.apiServer;
  [rows[4], rows[5]] = [rows[5]!, rows[4]!];
  const proof = verify(f),
    result = object(object(proof.owners[0]!.sourceSupplement).sourceCleanup);
  assert.equal(result.apiServer, rows);
  assert.deepEqual(
    rows.slice(4, 6).map((row) => object(row).id),
    [6, 5],
  );
});
test("historical source/SDK2 and wrong image/observer/helper/input never acquire cleanup authority", () => {
  changesRefuse([
    (f) => {
      f.sourceRecord.role = f.context.role = "before";
    },
    (f) => {
      f.sourceRecord.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      f.sourceRecord.observerSha256 = "0".repeat(64);
    },
    ...[observer, helper, input].map((file) => (f: Fixture) => {
      f.files.set(file, Buffer.from("changed"));
    }),
    (f) => {
      f.sourceRecord.observerDependencies = [];
    },
    (f) => {
      object(f.sourceRecord.attached).observerInputDependencies = [];
    },
    (f) => {
      f.files.set(
        "Cargo.toml",
        Buffer.from(
          Buffer.from(f.files.get("Cargo.toml")!).toString().replaceAll("beta.4", "beta.2"),
        ),
      );
    },
  ]);
});
test("root pins, every checksum/dependency vector and seven actual public compiler libraries are required", () => {
  changesRefuse([
    (f) => {
      f.files.set(
        "Cargo.toml",
        Buffer.from(
          Buffer.from(f.files.get("Cargo.toml")!)
            .toString()
            .replace("=2.0.0-beta.4", "^2.0.0-beta.4"),
        ),
      );
    },
    (f) => {
      f.files.set(
        "Cargo.toml",
        Buffer.from(
          Buffer.from(f.files.get("Cargo.toml")!).toString() +
            '\n[patch.crates-io]\ncorsa = { path = "local" }\n',
        ),
      );
    },
    ...sdk4CleanupPackages.map(([, checksum]) => (f: Fixture) => {
      f.files.set(
        "Cargo.lock",
        Buffer.from(
          Buffer.from(f.files.get("Cargo.lock")!).toString().replace(checksum, "0".repeat(64)),
        ),
      );
    }),
    (f) => {
      f.files.set(
        "Cargo.lock",
        Buffer.from(
          Buffer.from(f.files.get("Cargo.lock")!).toString().replace('"corsa_client",', '"fake",'),
        ),
      );
    },
    (f) => {
      f.artifacts.splice(0, 1);
    },
    (f) => {
      f.artifacts[0]!.package_id = "path+file:///fake#corsa@2.0.0-beta.4";
    },
    (f) => {
      f.artifacts[0]!.manifest_path = String(f.artifacts[0]!.manifest_path).replace(
        "2.0.0",
        "2x0x0",
      );
    },
    (f) => {
      f.artifacts[0]!.package_id = registry + "#corsa@2.0.0-beta.2";
    },
    (f) => {
      f.artifacts.unshift(f.artifacts.pop()!);
    },
  ]);
});
test("same source/group/context binding cannot be copied, changed or reused for a different owner view", () => {
  const f = fixture(),
    bound = authority(f);
  assert.ok(bound);
  assert.throws(() => verify(f, structuredClone(bound)));
  assert.throws(() =>
    verifyBetaLogicalRoute(
      f.owners,
      f.context,
      f.source,
      f.root,
      structuredClone(f.phases),
      f.attached,
      bound,
    ),
  );
  bound.source = "0".repeat(40);
  assert.throws(() => verify(f, bound));
});
test("early, unknown, duplicate, reordered or main closes and extra queries refuse", () => {
  changesRefuse([
    (f) => {
      const rows = [...f.owners[0]!.client],
        close = rows.findIndex((row) => object(row).method === "textDocument/didClose");
      const [value] = rows.splice(close, 1);
      rows.splice(
        rows.findIndex((row) => object(row).id === 55),
        0,
        value!,
      );
      f.owners[0]!.client = rows;
    },
    (f) => {
      const rows = f.owners[0]!.client,
        closes = rows.filter((row) => object(row).method === "textDocument/didClose");
      object(object(object(closes[0]).params).textDocument).uri = f.attached.supportUri;
    },
    (f) => {
      const rows = f.owners[0]!.client,
        first = rows.find((row) => object(row).method === "textDocument/didClose")!;
      rows.splice(rows.length - 1, 0, structuredClone(first));
    },
    (f) => {
      const row = f.owners[0]!.client.find(
        (row) => object(row).method === "textDocument/didClose",
      )!;
      object(object(object(row).params).textDocument).uri = f.attached.mainUri;
    },
    (f) => {
      const row = f.owners[0]!.client.find(
        (row) => object(row).method === "textDocument/didClose",
      )!;
      object(object(object(row).params).textDocument).uri = "file:///other/unknown.ts";
    },
    (f) => {
      const rows = f.owners[0]!.client,
        a = rows.findIndex((row) => object(row).id === 56),
        b = rows.findIndex((row) => object(row).id === 57);
      [rows[a], rows[b]] = [rows[b]!, rows[a]!];
    },
    (f) => {
      f.owners[0]!.client.splice(-1, 0, {
        jsonrpc: "2.0",
        id: 1000,
        method: "textDocument/documentSymbol",
        params: { textDocument: { uri: f.attached.mainUri } },
      });
      f.owners[0]!.server.splice(-1, 0, { jsonrpc: "2.0", id: 1000, result: [] });
    },
  ]);
});
test("cutoff is source55, never self-selected raw length or coherent shortened/extended vectors", () => {
  changesRefuse([
    (f) => {
      f.owners[0]!.client = f.owners[0]!.client.filter((row) => object(row).id !== 54);
      f.owners[0]!.server = f.owners[0]!.server.filter((row) => object(row).id !== 54);
    },
    (f) => {
      const row = f.owners[0]!.client.find((row) => object(row).id === 55)!,
        response = f.owners[0]!.server.find((row) => object(row).id === 55)!;
      f.owners[0]!.client.splice(-1, 0, { ...object(row), id: 1001 });
      f.owners[0]!.server.splice(-1, 0, { ...object(response), id: 1001 });
    },
    (f) => {
      object(f.owners[0]!.server.find((row) => object(row).id === 4)).result = { wrong: "phase" };
    },
    (f) => {
      object(f.owners[0]!.server.find((row) => object(row).id === 55)).result = {
        kind: "full",
        items: [],
      };
    },
  ]);
});
test("whole readiness/shutdown params, response vectors and source URI order stay closed", () => {
  changesRefuse([
    (f) => {
      object(object(f.owners[0]!.client.find((row) => object(row).id === 56)).params).extra = true;
    },
    (f) => {
      object(
        object(object(f.owners[0]!.client.find((row) => object(row).id === 56)).params)
          .textDocument,
      ).uri = f.attached.supportUri;
    },
    (f) => {
      object(f.owners[0]!.server.find((row) => object(row).id === 56)).result = [];
    },
    (f) => {
      f.owners[0]!.server = f.owners[0]!.server.filter((row) => object(row).id !== 57);
    },
    (f) => {
      object(f.owners[0]!.server.find((row) => object(row).id === 58)).result = {};
    },
  ]);
});
test("API26 full metadata/handles/config/dependencies cannot be elided or replaced by semantic API", () => {
  changesRefuse([
    (f) => {
      object(object(f.owners[0]!.apiServer[0]).result).currentDirectory = "/other/native-owner";
    },
    (f) => {
      object(f.owners[0]!.apiClient[0]).params = {};
    },
    (f) => {
      object(object(f.owners[0]!.apiServer[1]).result).errors = [{ code: 1 }];
    },
    (f) => {
      object(f.owners[0]!.apiServer[4]).result = {};
      delete object(f.owners[0]!.apiServer[4]).error;
    },
    (f) => {
      object(object(f.owners[0]!.apiClient[7]).params).snapshot = 999;
    },
    (f) => {
      object(object(f.owners[0]!.apiServer[6]).result).snapshot = 100;
    },
    (f) => {
      f.owners[0]!.apiClient.push({
        jsonrpc: "2.0",
        id: 100,
        method: "getSemanticDiagnostics",
        params: {},
      });
      f.owners[0]!.apiServer.push({ jsonrpc: "2.0", id: 100, result: [] });
    },
    (f) => {
      const rows = f.owners[0]!.apiServer;
      [rows[6], rows[7]] = [rows[7]!, rows[6]!];
    },
  ]);
});
test("the rejected1341 premature Support close/reopen stays rejected even with SDK4 custody", () => {
  const f = fixture(),
    client = f.owners[0]!.client;
  const open = client.find(
    (row) => object(row).method === "textDocument/didOpen" && uri(row) === f.attached.supportUri,
  )!;
  const index = client.findIndex((row) => object(row).id === 54);
  client.splice(
    index,
    0,
    {
      jsonrpc: "2.0",
      method: "textDocument/didClose",
      params: { textDocument: { uri: f.attached.supportUri } },
    },
    structuredClone(open),
  );
  assert.throws(() => verify(f));
});
