/** Synthetic filesystem custody tests; these are not native graph/performance evidence. */
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
  statSync,
  symlinkSync,
  unlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { createNativeGraphArchive } from "./typechecker-native-graph-archive.mjs";

const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
function fixture() {
  const root = mkdtempSync(join(tmpdir(), "native-graph-custody-"));
  const directory = join(root, "archive");
  mkdirSync(directory);
  const config = join(root, "tsconfig.json");
  const runtime = join(root, "fake-native");
  const first = join(root, "z 雪.ts");
  const second = join(root, "a.ts");
  const content = Buffer.from([0x65, 0x78, 0x70, 0x6f, 0x72, 0x74, 0xff, 0x0a]);
  writeFileSync(first, content);
  writeFileSync(second, content);
  writeFileSync(runtime, "fake-native-runtime\n");
  const configBytes = Buffer.from(
    '{\n  "compilerOptions": {"rootDirs":["z","a"]},\n  "files":["z 雪.ts","a.ts","z 雪.ts"], "include":["z*","a*","z*"], "exclude":[]\n}\n',
  );
  writeFileSync(config, configBytes);
  const stem = join(directory, "capture");
  const args = ["--project", config, "--checkers", "1"];
  const options = { directory, stem, cwd: root, config, configBytes, runtime, args };
  const members = (paths = [first, second, first]) =>
    paths.map((path) => {
      const bytes = readFileSync(path);
      return { path, bytes: bytes.length, sha256: sha(bytes) };
    });
  const list = (paths = [first, second, first], phase = "before", ending = "\n") => {
    writeFileSync(`${stem}.membership-${phase}.stdout.txt`, paths.join(ending) + ending);
    writeFileSync(`${stem}.membership-${phase}.stderr.txt`, "");
  };
  const start = (paths = [first, second, first]) => {
    list(paths);
    const archive = createNativeGraphArchive(options);
    archive.captureMembers(members(paths));
    return archive;
  };
  const finish = (archive, paths = [first, second, first]) => {
    list(paths, "after");
    return archive.finish(members(paths));
  };
  return { root, first, second, config, runtime, options, members, list, start, finish };
}

async function withFixture(action) {
  const value = fixture();
  try {
    await action(value);
  } finally {
    rmSync(value.root, { recursive: true, force: true });
  }
}

await test("raw bytes and duplicate/root/member order survive content-addressed dedup", async () => {
  await withFixture((f) => {
    const archive = f.start();
    const record = f.finish(archive);
    assert.deepEqual(record.args, f.options.args);
    assert.equal(record.cwd, f.root);
    assert.equal(record.graphClosureClaimed, false);
    assert.deepEqual(record.config.declaredRootSelection.files, ["z 雪.ts", "a.ts", "z 雪.ts"]);
    assert.deepEqual(record.config.declaredRootSelection.include, ["z*", "a*", "z*"]);
    assert.deepEqual(
      record.membersBefore.map((m) => m.reportedPath),
      [f.first, f.second, f.first],
    );
    assert.deepEqual(record.membersAfter, record.membersBefore);
    assert.equal(new Set(record.membersBefore.map((m) => m.object)).size, 1);
    assert(
      readFileSync(join(f.options.directory, record.config.rawFile)).equals(f.options.configBytes),
    );
    for (const member of record.membersBefore)
      assert(
        readFileSync(join(f.options.directory, member.object)).equals(readFileSync(member.path)),
      );
    assert.equal(record.listings.before.stdout.sha256, record.listings.after.stdout.sha256);
    assert.throws(() => f.finish(archive), /before membership|finish once/u);
  });
});

await test("same-byte members still fail when native ordering changes", async () => {
  await withFixture((f) => {
    const archive = f.start();
    assert.throws(() => f.finish(archive, [f.second, f.first, f.first]), /order\/bytes\/identity/u);
    assert.equal(existsSync(`${f.options.stem}.graph.json`), false);
  });
});

await test("separate row directories share source objects but retain local config and streams", async () => {
  await withFixture((f) => {
    const objectsDirectory = join(f.root, "shared-objects");
    const records = ["row-one", "row-two"].map((row) => {
      const directory = join(f.root, row);
      mkdirSync(directory);
      const stem = join(directory, "capture");
      for (const phase of ["before", "after"]) {
        writeFileSync(
          `${stem}.membership-${phase}.stdout.txt`,
          `${f.first}\n${f.second}\n${f.first}\n`,
        );
        writeFileSync(`${stem}.membership-${phase}.stderr.txt`, "");
      }
      const archive = createNativeGraphArchive({ ...f.options, directory, stem, objectsDirectory });
      archive.captureMembers(f.members());
      const record = archive.finish(f.members());
      assert.equal(record.objectStore.path, realpathSync.native(objectsDirectory));
      assert.equal(
        realpathSync.native(join(directory, record.objectStore.relativeDirectory)),
        record.objectStore.path,
      );
      assert(readFileSync(join(directory, record.config.rawFile)).equals(f.options.configBytes));
      for (const member of record.membersBefore)
        assert(readFileSync(join(directory, member.object)).equals(readFileSync(member.path)));
      assert.equal(existsSync(join(directory, "graph-objects")), false);
      return record;
    });
    assert.deepEqual(records[0].membersBefore, records[1].membersBefore);
    const hash = sha(readFileSync(f.first));
    assert.deepEqual(readdirSync(join(objectsDirectory, hash.slice(0, 2))), [hash]);
  });
});

await test("same-length restored-mtime source edits fail", async () => {
  await withFixture((f) => {
    const archive = f.start();
    const stat = statSync(f.first);
    const changed = Buffer.from(readFileSync(f.first));
    changed[0] ^= 1;
    writeFileSync(f.first, changed);
    utimesSync(f.first, stat.atime, stat.mtime);
    assert.throws(() => f.finish(archive), /order\/bytes\/identity/u);
  });
});

await test("source replacement with identical bytes fails identity custody", async () => {
  await withFixture((f) => {
    const archive = f.start();
    const replacement = join(f.root, "replacement.ts");
    writeFileSync(replacement, readFileSync(f.first));
    renameSync(replacement, f.first);
    assert.throws(() => f.finish(archive), /order\/bytes\/identity/u);
  });
});

await test("config root reorder and native runtime replacement fail", async () => {
  for (const kind of ["config", "runtime"]) {
    await withFixture((f) => {
      const archive = f.start();
      if (kind === "config") {
        writeFileSync(
          f.config,
          readFileSync(f.config, "utf8")
            .replace('["z 雪.ts","a.ts","z 雪.ts"]', '["a.ts","z 雪.ts","z 雪.ts"]')
            .replace('["z*","a*","z*"]', '["a*","z*","z*"]'),
        );
      } else writeFileSync(f.runtime, "different-runtime!!\n");
      assert.throws(() => f.finish(archive), /config identity|runtime identity/u);
    });
  }
});

await test("leaf and ancestor symlink retargets fail even with identical bytes", async () => {
  for (const ancestor of [false, true]) {
    await withFixture((f) => {
      let path = join(f.root, "link.ts");
      let link = path;
      let oldTarget = f.first;
      let newTarget = f.second;
      if (ancestor) {
        oldTarget = join(f.root, "old");
        newTarget = join(f.root, "new");
        for (const dir of [oldTarget, newTarget]) {
          mkdirSync(dir);
          writeFileSync(join(dir, "source.ts"), readFileSync(f.first));
        }
        link = join(f.root, "scope");
        path = join(link, "source.ts");
      }
      symlinkSync(oldTarget, link);
      const archive = f.start([path]);
      unlinkSync(link);
      symlinkSync(newTarget, link);
      assert.throws(() => f.finish(archive, [path]), /order\/bytes\/identity/u);
    });
  }
});

await test("a nested symlink target spelling change fails despite the same resolved file", async () => {
  await withFixture((f) => {
    const inner = join(f.root, "inner");
    const outer = join(f.root, "outer");
    symlinkSync(f.first, inner);
    symlinkSync(inner, outer);
    const archive = f.start([outer]);
    unlinkSync(inner);
    symlinkSync("./z 雪.ts", inner);
    assert.throws(() => f.finish(archive, [outer]), /order\/bytes\/identity/u);
  });
});

await test("raw symlink/.. spelling selects the observed filesystem member", async () => {
  await withFixture((f) => {
    const target = join(f.root, "other");
    mkdirSync(join(target, "inner"), { recursive: true });
    const actual = join(target, "z 雪.ts");
    writeFileSync(actual, readFileSync(f.first));
    const link = join(f.root, "scope");
    symlinkSync(join(target, "inner"), link);
    const reportedPath = `${link}/../z 雪.ts`;
    f.list([reportedPath]);
    const archive = createNativeGraphArchive(f.options);
    const metadata = f.members([f.first]);
    archive.captureMembers(metadata);
    f.list([reportedPath], "after");
    const record = archive.finish(metadata);
    assert.equal(record.membersBefore[0].reportedPath, reportedPath);
    assert.equal(record.membersBefore[0].path, reportedPath);
    assert.equal(record.membersBefore[0].realPath, realpathSync.native(actual));
    assert.notEqual(record.membersBefore[0].realPath, realpathSync.native(f.first));
  });
});

await test("corrupt and symlinked content-addressed objects are rejected", async () => {
  for (const symlink of [false, true]) {
    await withFixture((f) => {
      const archive = f.start();
      const bytes = readFileSync(f.first);
      const object = join(f.options.directory, "graph-objects", sha(bytes).slice(0, 2), sha(bytes));
      if (symlink) {
        unlinkSync(object);
        symlinkSync(f.first, object);
      } else {
        chmodSync(object, 0o600);
        writeFileSync(object, "corrupted-object");
      }
      assert.throws(() => f.finish(archive), /archive object/u);
    });
  }
});

await test("listing byte changes and supplied hash drift are rejected", async () => {
  await withFixture((f) => {
    const archive = f.start();
    f.list(undefined, "after", "\r\n");
    assert.throws(() => archive.finish(f.members()), /membership stdout changed/u);
  });
  await withFixture((f) => {
    f.list();
    const archive = createNativeGraphArchive(f.options);
    const members = f.members();
    members[0].sha256 = "0".repeat(64);
    assert.throws(() => archive.captureMembers(members), /member bytes changed/u);
  });
});

await test("concurrent shard archives publish one completed shared content object", async () => {
  await withFixture(async (f) => {
    writeFileSync(f.first, Buffer.alloc(1024 * 1024, 0xa5));
    const helper = new URL("./typechecker-native-graph-archive.mjs", import.meta.url).href;
    const source = `
import { createNativeGraphArchive } from ${JSON.stringify(helper)};
import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
const options = JSON.parse(process.argv[1]);
const path = process.argv[2];
options.configBytes = readFileSync(options.config);
const bytes = readFileSync(path);
const members = [{ path, bytes: bytes.length, sha256: createHash("sha256").update(bytes).digest("hex") }];
for (const phase of ["before", "after"]) {
  writeFileSync(options.stem + ".membership-" + phase + ".stdout.txt", path + "\\n");
  writeFileSync(options.stem + ".membership-" + phase + ".stderr.txt", "");
}
const archive = createNativeGraphArchive(options);
archive.captureMembers(members);
archive.finish(members);
`;
    await Promise.all(
      [0, 1, 2, 3].map(
        (index) =>
          new Promise((resolve, reject) => {
            const options = { ...f.options, stem: join(f.options.directory, `shard${index}`) };
            const child = spawn(process.execPath, [
              "--input-type=module",
              "-e",
              source,
              JSON.stringify(options),
              f.first,
            ]);
            let stderr = "";
            child.stderr.on("data", (bytes) => {
              stderr += bytes;
            });
            child.on("error", reject);
            child.on("close", (code) =>
              code === 0 ? resolve(undefined) : reject(new Error(stderr)),
            );
          }),
      ),
    );
    const hash = sha(readFileSync(f.first));
    const objects = join(f.options.directory, "graph-objects", hash.slice(0, 2));
    assert.deepEqual(readdirSync(objects), [hash]);
    for (const index of [0, 1, 2, 3]) {
      const record = JSON.parse(
        readFileSync(join(f.options.directory, `shard${index}.graph.json`), "utf8"),
      );
      assert.equal(record.membersAfter[0].sha256, hash);
      assert(
        readFileSync(join(f.options.directory, record.membersAfter[0].object)).equals(
          readFileSync(f.first),
        ),
      );
    }
  });
});
