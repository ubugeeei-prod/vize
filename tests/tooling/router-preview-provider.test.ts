import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { buildProvider } from "./support/router-preview/build-provider.ts";
import { fixtureDirectory, readAuthority } from "./support/router-preview/authority.ts";
import { readProjectFixture } from "./support/router-preview/project.ts";

const digest = (value: string) => createHash("sha256").update(value).digest("hex");

test("preview identity pins real source rather than reusing the published version number", () => {
  const { authority } = readAuthority();
  assert.equal(authority.sourceInputs.length, 9);
  assert.deepEqual(authority.qualification, {
    state: "discovery",
    sourceBuiltPackageIsNotPublishedNpmArchive: true,
    automaticProjection: "not-qualified",
  });
  const project = readProjectFixture();
  assert.deepEqual(
    [...project.files.keys()],
    [
      "generate.mjs",
      "package.json",
      "src/pages/index.vue",
      "src/pages/products/[id=int].vue",
      "src/params/date.ts",
      "src/router/index.ts",
      "tsconfig.json",
    ],
  );
  assert.equal(project.authority.strictOracles, "not-frozen");
});

test("preview project input drift is refused before installation", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preview-input-control-"));
  try {
    fs.cpSync(fixtureDirectory, root, { recursive: true });
    fs.appendFileSync(path.join(root, "project/src/pages/products/[id=int].vue.txt"), "\n");
    const expected = readProjectFixture().authority.files.find(
      (entry) => entry.path === "src/pages/products/[id=int].vue",
    )!;
    assert.throws(() => readProjectFixture(root), {
      code: "ERR_ASSERTION",
      actual: expected.bytes + 1,
      expected: expected.bytes,
      operator: "strictEqual",
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the real provider build cannot be started by a local helper invocation", () => {
  const previous = process.env.GITHUB_ACTIONS;
  delete process.env.GITHUB_ACTIONS;
  try {
    assert.throws(() => buildProvider("/unused-source", "/unused-output"), {
      code: "ERR_ASSERTION",
      actual: undefined,
      expected: "true",
      operator: "strictEqual",
    });
  } finally {
    if (previous === undefined) delete process.env.GITHUB_ACTIONS;
    else process.env.GITHUB_ACTIONS = previous;
  }
});

test("archive custody keeps complete identities and rejects traversal, links, and duplicates", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preview-archive-control-"));
  const builder = `import io,sys,tarfile
p,kind=sys.argv[1:]
with tarfile.open(p,'w:gz') as t:
 d=tarfile.TarInfo('package/');d.type=tarfile.DIRTYPE;t.addfile(d)
 entries=[('package/package.json','{"name":"owned-control"}\\n'),('package/dist/index.d.ts','export type Value = number\\n')]
 if kind=='traversal':entries[0]=('package/../outside.txt','x')
 if kind=='absolute':entries[0]=('/package/outside.txt','x')
 if kind=='foreign':entries[0]=('foreign/outside.txt','x')
 if kind=='duplicate':entries.append(entries[0])
 for name,value in entries:
  b=value.encode();m=tarfile.TarInfo(name);m.size=len(b);t.addfile(m,io.BytesIO(b))
 if kind in ('symlink','hardlink'):
  m=tarfile.TarInfo('package/linked');m.type=tarfile.SYMTYPE if kind=='symlink' else tarfile.LNKTYPE;m.linkname='../../outside';t.addfile(m)
`;
  try {
    for (const kind of [
      "regular",
      "traversal",
      "absolute",
      "foreign",
      "duplicate",
      "symlink",
      "hardlink",
    ]) {
      const archive = path.join(root, `${kind}.tgz`);
      const create = spawnSync("python3", ["-c", builder, archive, kind], { encoding: "utf8" });
      assert.equal(create.error, undefined);
      assert.equal(create.status, 0, create.stderr);
      const read = spawnSync(
        "python3",
        [path.join(fixtureDirectory, "archive_inventory.py"), archive],
        { encoding: "utf8" },
      );
      assert.equal(read.error, undefined);
      if (kind === "regular") {
        assert.equal(read.status, 0, read.stderr);
        assert.equal(read.stderr, "");
        assert.deepEqual(JSON.parse(read.stdout), [
          { path: "dist/index.d.ts", bytes: 27, sha256: digest("export type Value = number\n") },
          { path: "package.json", bytes: 25, sha256: digest('{"name":"owned-control"}\n') },
        ]);
      } else {
        assert.equal(read.status, 1, kind);
        assert.equal(read.stdout, "", kind);
        const expected = {
          traversal: "unsafe package member: 'package/../outside.txt'",
          absolute: "unsafe package member: '/package/outside.txt'",
          foreign: "unsafe package member: 'foreign/outside.txt'",
          duplicate: "duplicate package member: 'package.json'",
          symlink: "non-regular package member: 'package/linked'",
          hardlink: "non-regular package member: 'package/linked'",
        };
        assert.deepEqual(
          JSON.parse(read.stderr),
          { error: expected[kind as keyof typeof expected] },
          kind,
        );
      }
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
