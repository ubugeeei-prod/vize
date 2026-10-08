const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const before = path.resolve(
  __dirname,
  "../../../tests/_fixtures/differential/lint/n8n-native-custody-8142/native-custody-before.cjs.txt",
);
const after = path.join(__dirname, "n8n-native-custody.cjs");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const source = '<template><p v-html="value" /></template>\n';
// Byte-exact Vize observer source from genuine #8212 head 8ec57bb9.
assert.equal(
  digest(fs.readFileSync(before)),
  "c7fcff7497d2ebadbc1565e88663e1feb5a17b6ff288fa47ffe77092c8993338",
);

function harness(code, directory, intercept = {}) {
  fs.mkdirSync(directory, { recursive: true });
  const binary = path.join(directory, "vize-vitrine.mock.node");
  const filename = path.join(directory, "Original.vue");
  const calls = path.join(directory, "calls.jsonl");
  const sources = path.join(directory, "sources");
  fs.rmSync(sources, { recursive: true, force: true });
  fs.rmSync(calls, { force: true });
  fs.writeFileSync(binary, "not a native addon: recorder unit control only");
  fs.writeFileSync(filename, source);
  const custody = {
    schema: "vize.oxlint.n8n-source-native",
    version: 1,
    source: { head: "a".repeat(40), tree: "b".repeat(40) },
    binary: { path: binary, sha256: digest(fs.readFileSync(binary)) },
    calls,
    sources,
  };
  const custodyPath = path.join(directory, "custody.json");
  fs.writeFileSync(custodyPath, JSON.stringify(custody));
  const metrics = { native: 0, opens: [], closes: [], writes: [] };
  const handlers = [];
  const io = { ...fs };
  io.openSync = (...args) => {
    const fd = fs.openSync(...args);
    if (args[0] === calls) metrics.opens.push({ fd, flags: args[1] });
    return fd;
  };
  io.closeSync = (fd) => {
    metrics.closes.push(fd);
    fs.closeSync(fd);
  };
  io.appendFileSync = (...args) => {
    metrics.writes.push(args[0]);
    intercept.append?.(...args);
    return fs.appendFileSync(...args);
  };
  io.readFileSync = (...args) => {
    intercept.read?.(...args);
    return fs.readFileSync(...args);
  };
  const moduleApi = {
    _extensions: {
      ".node": (loaded) => {
        loaded.exports = {
          lintPatinaSfc: (text, options) => {
            metrics.native++;
            if (options.fail) throw new Error("complete native failure");
            return {
              diagnostics: [{ message: text, labels: [1, 1], unknown: { options } }],
              filename: options.filename,
              errorCount: 0,
              warningCount: 1,
              unknown: ["retain", null, { nested: true }],
            };
          },
        };
      },
    },
  };
  vm.runInNewContext(fs.readFileSync(code, "utf8"), {
    Buffer,
    require: (name) =>
      name === "node:fs" ? io : name === "node:module" ? moduleApi : require(name),
    process: {
      pid: process.pid,
      env: {
        GITHUB_ACTIONS: "true",
        GITHUB_SHA: custody.source.head,
        VIZE_N8N_NATIVE_CUSTODY: custodyPath,
        VIZE_N8N_REPLAY_PHASE: "authored-recorder-law",
      },
      once: (event, handler) => {
        assert.equal(event, "exit");
        handlers.push(handler);
      },
    },
  });
  const loaded = {};
  moduleApi._extensions[".node"](loaded, binary);
  return {
    ...custody,
    filename,
    metrics,
    lint: (options = {}) => loaded.exports.lintPatinaSfc(source, { filename, ...options }),
    bytes: () => fs.readFileSync(calls),
    finish: () => handlers.forEach((handler) => handler()),
  };
}

if (process.argv[2] === "terminate") {
  const h = harness(after, process.argv[3]);
  h.lint({ enabledRules: ["vue/no-v-html"], unknown: { complete: true } });
  process.kill(process.pid, "SIGTERM");
} else {
  void test("whole old/new records retain every argument/result, throw and order byte-exact", () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-recorder-equivalence-"));
    try {
      const vectors = [];
      for (const code of [before, after]) {
        const h = harness(code, directory);
        try {
          for (let index = 0; index < 256; index++)
            h.lint({ enabledRules: ["vue/no-v-html"], index, nested: { value: [1, null] } });
          assert.throws(() => h.lint({ fail: true }), /complete native failure/);
          vectors.push(h.bytes());
          assert.equal(h.metrics.native, 257);
          if (code === after) {
            assert.equal(h.metrics.opens.length, 1);
            assert.equal(h.metrics.opens[0].flags, "a");
            assert.ok(h.metrics.writes.every((fd) => fd === h.metrics.opens[0].fd));
          }
        } finally {
          h.finish();
        }
        if (code === after) assert.deepEqual(h.metrics.closes, [h.metrics.opens[0].fd]);
      }
      assert.ok(vectors[0].equals(vectors[1]));
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });

  for (const code of [before, after]) {
    void test(`${path.basename(code)}: physical and repeated CAS mutations fail before native`, () => {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-recorder-mutation-"));
      const h = harness(code, directory);
      try {
        h.lint();
        fs.writeFileSync(path.join(h.sources, digest(Buffer.from(source)) + ".vue"), "mutated");
        assert.throws(() => h.lint(), /source digest collision or mutation/);
        assert.equal(h.metrics.native, 1);
        fs.writeFileSync(h.filename, "foreign original");
        assert.throws(() => h.lint(), /native must lint complete original bytes/);
        assert.equal(h.metrics.native, 1);
        assert.equal(h.bytes().toString().trim().split("\n").length, 2);
      } finally {
        h.finish();
        fs.rmSync(directory, { recursive: true, force: true });
      }
    });
  }

  void test("exclusive CAS creation races retain exact bytes and refuse foreign bytes", () => {
    for (const raceBytes of [source, "foreign collision"]) {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-recorder-race-"));
      const cas = path.join(directory, "sources", digest(Buffer.from(source)) + ".vue");
      let raced = false;
      const h = harness(after, directory, {
        read(filename) {
          if (filename !== cas || raced) return;
          raced = true;
          fs.writeFileSync(cas, raceBytes, { flag: "wx" });
          const error = new Error("creation raced after missing lookup");
          error.code = "ENOENT";
          throw error;
        },
      });
      try {
        if (raceBytes === source) {
          h.lint();
          assert.equal(h.metrics.native, 1);
        } else {
          assert.throws(() => h.lint(), /source digest collision or mutation/);
          assert.equal(h.metrics.native, 0);
        }
        assert.equal(fs.readFileSync(cas, "utf8"), raceBytes);
      } finally {
        h.finish();
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  });

  void test("a failed append aborts and closes the owned descriptor without false records", () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-recorder-append-"));
    let appends = 0;
    const h = harness(after, directory, {
      append() {
        if (++appends < 2) return;
        const error = new Error("authored append failure");
        error.code = "EIO";
        throw error;
      },
    });
    try {
      assert.throws(() => h.lint(), /authored append failure/);
      assert.equal(h.metrics.native, 1);
      assert.equal(h.bytes().toString().trim().split("\n").length, 1);
    } finally {
      h.finish();
      assert.deepEqual(h.metrics.closes, [h.metrics.opens[0].fd]);
      assert.throws(() => fs.fstatSync(h.metrics.opens[0].fd), { code: "EBADF" });
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });

  void test("SIGTERM preserves synchronous complete records without granting completion", () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-recorder-abort-"));
    try {
      const result = spawnSync(process.execPath, [__filename, "terminate", directory], {
        timeout: 5_000,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, "SIGTERM");
      assert.equal(result.status, null);
      const bytes = fs.readFileSync(path.join(directory, "calls.jsonl"));
      assert.ok(bytes.toString().endsWith("\n"));
      const events = bytes.toString().trim().split("\n").map(JSON.parse);
      assert.equal(events.length, 2);
      assert.equal(events[0].kind, "load");
      assert.equal(events[1].kind, "call");
      assert.equal(events[1].outcome, "return");
      assert.equal(events.filter((row) => row.kind === "call").length, 1);
      // An aborted process has only its actual completed calls, never an
      // invented second packet that could satisfy the campaign's exact count.
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
