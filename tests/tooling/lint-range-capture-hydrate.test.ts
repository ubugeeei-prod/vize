import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  attestRawCheckout,
  firstRawCheckout,
  rawRegistryAttributes,
} from "../../tools/support/ci/lint-range-capture/hydrate.ts";
import { verifyTrackedSource } from "../../tools/support/ci/lint-range-capture/preflight.ts";
import { decodeFrames } from "../../tools/support/ci/lint-range-capture/codec.ts";
import { projects } from "../../tools/support/ci/lint-range-capture/projects.ts";

test("raw first fixture checkout retains original CRLF blobs and strict later mutation refusal", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-lint-raw-first-"));
  const invoke = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-C", cwd, ...args], { stdio: ["ignore", "pipe", "pipe"] });
  const commit = (cwd: string) =>
    invoke(
      cwd,
      "-c",
      "user.name=Capture Law",
      "-c",
      "user.email=capture@example.invalid",
      "commit",
      "-qm",
      "original fixture",
    );
  try {
    const repository = join(root, "original");
    mkdirSync(repository);
    invoke(repository, "init", "-q");
    const original = new Map([
      [
        ".gitattributes",
        Buffer.from(
          "apps/v4/styles/reka-*/** text eol=lf\napps/v4/public/r/**/*.json text eol=lf\n" +
            "apps/v4/registry/**/*.json text eol=lf\n",
        ),
      ],
      ["apps/v4/public/r/colors/gray.json", Buffer.from('{"original":"registry"}\r\n')],
      ["apps/v4/registry/entry.json", Buffer.from('{"original":"entry"}\r\n')],
      ["apps/v4/styles/reka-a/theme.css", Buffer.from("original-style\n")],
      ["apps/App.vue", Buffer.from("<template>original Vue</template>\n")],
    ]);
    for (const [path, bytes] of original) {
      const blob = execFileSync("git", ["-C", repository, "hash-object", "-w", "--stdin"], {
        input: bytes,
      })
        .toString()
        .trim();
      invoke(repository, "update-index", "--add", "--cacheinfo", "100644," + blob + "," + path);
    }
    const target = "App.vue";
    const linkBlob = execFileSync("git", ["-C", repository, "hash-object", "-w", "--stdin"], {
      input: Buffer.from(target),
    })
      .toString()
      .trim();
    invoke(
      repository,
      "update-index",
      "--add",
      "--cacheinfo",
      "120000," + linkBlob + ",apps/link.vue",
    );
    commit(repository);
    const revision = invoke(repository, "rev-parse", "HEAD").toString().trim();
    const ordinary = join(root, "ordinary");
    execFileSync("git", ["clone", "-q", repository, ordinary]);
    for (const [path, bytes] of original)
      assert.deepEqual(readFileSync(join(ordinary, path)), bytes);
    assert.match(invoke(ordinary, "diff", "--name-only", "HEAD").toString(), /gray\.json/u);

    const parent = join(root, "capture"),
      fixture = projects[3].fixturePath;
    mkdirSync(parent);
    invoke(parent, "init", "-q");
    writeFileSync(
      join(parent, ".gitmodules"),
      `[submodule "${fixture}"]\n\tpath = ${fixture}\n\turl = ${repository}\n`,
    );
    invoke(parent, "add", ".gitmodules");
    invoke(parent, "update-index", "--add", "--cacheinfo", "160000," + revision + "," + fixture);
    commit(parent);
    const expected = invoke(parent, "rev-parse", "HEAD").toString().trim();
    const checkout = join(parent, fixture);
    mkdirSync(join(parent, "tests/_fixtures/_git"), { recursive: true });
    invoke(parent, "submodule", "init", "--", fixture);
    mkdirSync(checkout);
    invoke(checkout, "init", "--quiet");
    invoke(checkout, "remote", "add", "origin", repository);
    invoke(checkout, "fetch", "--depth", "1", "--no-tags", "origin", revision);
    firstRawCheckout(checkout, revision);
    invoke(parent, "submodule", "absorbgitdirs", "--", fixture);
    assert.equal(
      readFileSync(
        invoke(checkout, "rev-parse", "--git-path", "info/attributes").toString().trim(),
        "utf8",
      ),
      rawRegistryAttributes,
    );
    for (const [path, bytes] of original)
      assert.deepEqual(readFileSync(join(checkout, path)), bytes);
    assert.equal(invoke(checkout, "diff", "--name-only", "HEAD").length, 0);
    verifyTrackedSource(parent, expected);
    const out = join(parent, "eslint-invalid-range-capture");
    const receipt = attestRawCheckout(checkout, revision, out);
    assert.equal(receipt.files.length, original.size + 1);
    assert.equal(receipt.allOriginalBlobsExact, true);
    assert.throws(() => firstRawCheckout(checkout, revision), /unmaterialized fixture/u);
    writeFileSync(
      join(checkout, "apps/v4/public/r/colors/gray.json"),
      Buffer.concat([original.get("apps/v4/public/r/colors/gray.json")!, Buffer.from("x")]),
    );
    assert.throws(() => verifyTrackedSource(parent, expected), /Tracked source has edits/u);
    const failedOut = join(root, "changed-attestation");
    assert.throws(
      () => attestRawCheckout(checkout, revision, failedOut),
      /Physical fixture bytes differ/u,
    );
    assert.equal(
      JSON.parse(readFileSync(join(failedOut, "shadcn-raw-checkout.json"), "utf8"))
        .allOriginalBlobsExact,
      false,
    );
    const frames = execFileSync(process.execPath, [
      fileURLToPath(
        new URL("../../tools/support/ci/lint-range-capture/frames.ts", import.meta.url),
      ),
      parent,
      expected,
    ]).toString();
    const decoded = decodeFrames(frames, expected);
    for (const name of ["shadcn-raw-checkout.json", "shadcn-tree.stdout.log", "source-drift.json"])
      assert.deepEqual(decoded.files.get(name), readFileSync(join(out, name)));
    assert.equal(decoded.header.actualProcessExit, null);
    assert.equal(decoded.header.acceptance, false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("lint capture checkout retains the pinned reporter through the actual five-commit boundary", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-lint-capture-ancestry-"));
  const invoke = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-C", cwd, ...args], { stdio: ["ignore", "pipe", "pipe"] });
  try {
    const repository = join(root, "source");
    mkdirSync(repository);
    invoke(repository, "init", "-q");
    let reporter = "";
    for (const message of [
      "reporter",
      "capture",
      "drift retention",
      "checkout correction",
      "raw fixture",
    ]) {
      writeFileSync(join(repository, "source.txt"), message + "\n");
      invoke(repository, "add", ".");
      invoke(
        repository,
        "-c",
        "user.name=Capture Law",
        "-c",
        "user.email=capture@example.invalid",
        "commit",
        "-qm",
        message,
      );
      if (!reporter) reporter = invoke(repository, "rev-parse", "HEAD").toString().trim();
    }
    const workflow = readFileSync(
      new URL("../../.github/workflows/lint-range-capture.yml", import.meta.url),
      "utf8",
    );
    const depth = Number(workflow.match(/fetch-depth: (\d+)/u)?.[1]);
    assert.equal(depth, 0);
    for (const value of [2, 4, depth]) {
      const checkout = join(root, "checkout-" + value);
      execFileSync("git", [
        "clone",
        "-q",
        ...(value ? ["--depth", String(value)] : []),
        pathToFileURL(repository).href,
        checkout,
      ]);
      const ancestry = () => invoke(checkout, "merge-base", "--is-ancestor", reporter, "HEAD");
      if (value) assert.throws(ancestry, /Not a valid commit name/u);
      else ancestry();
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
