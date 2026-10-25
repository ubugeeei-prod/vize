import assert from "node:assert/strict";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  createExperimentalRouterWorkspace,
  experimentalRouterDocumentTransport,
  normalizeExperimentalRouterPackets,
} from "./support/router-experimental-transport.ts";

const project = "/tmp/route-雪😀";
const references = ["typed-router.d.ts", "extra.d.ts"];
const header = references.map((file) => `/// <reference path="${project}/${file}" />\n`).join("");
const body = `const authored = "${project}/keep-this-string";\nconst route = { params: { page: 1 } };\nroute.params.page\n`;
const source = header + body;
const expectedSource =
  references.map((file) => `/// <reference path="$PROJECT/${file}" />\n`).join("") + body;
const position = Buffer.byteLength(source.slice(0, source.lastIndexOf("page")));
const expectedPosition = Buffer.byteLength(
  expectedSource.slice(0, expectedSource.lastIndexOf("page")),
);

test("reference transport counts every proven UTF8 span and preserves authored root literals", () => {
  assert.equal(project.length, 14);
  assert.equal(Buffer.byteLength(project), 18);
  const transport = experimentalRouterDocumentTransport(source, project, references);
  assert.equal(transport.normalizedSource, expectedSource);
  assert.equal(transport.relocate(position), expectedPosition);
  for (const delta of [-1, 1]) {
    assert.equal(transport.relocate(position + delta), expectedPosition + delta);
    assert.throws(
      () => assert.equal(transport.relocate(position + delta), expectedPosition),
      assert.AssertionError,
    );
  }
});

test("foreign roots and prefixes cannot acquire transport authority", () => {
  for (const foreign of [`${project}-other`, `/foreign${project}`, "/tmp/route-雪"])
    assert.throws(
      () => experimentalRouterDocumentTransport(source, foreign, references),
      assert.AssertionError,
    );
  assert.throws(
    () =>
      experimentalRouterDocumentTransport(
        source.replace(`${project}/typed-router.d.ts`, `${project}-other/typed-router.d.ts`),
        project,
        references,
      ),
    assert.AssertionError,
  );
  const transport = experimentalRouterDocumentTransport(
    source.replace("page: 1", "page: 2"),
    project,
    references,
  );
  assert.throws(
    () => assert.equal(transport.normalizedSource, expectedSource),
    assert.AssertionError,
  );
});

test("whole packet transport relocates exact path fields while preserving arbitrary text and extra fields", () => {
  const oneHeader = `/// <reference path="${project}/typed-router.d.ts" />\n`;
  const generatedSource = oneHeader + body;
  const generated = {
    file: "/tmp/private-owner/src/pages/products/[id=int].vue.ts",
    source: generatedSource,
    sha256: "captured-separately",
  };
  const offset = Buffer.byteLength(generatedSource.slice(0, generatedSource.lastIndexOf("page")));
  const packet = {
    uri: pathToFileURL(`${project}/src/Page.vue`).href,
    contents: `${project}/authored`,
    data: { fileName: generated.file, name: "page", position: offset },
    revision: 3,
    extra: { uri: pathToFileURL(`${project}-other/src/Page.vue`).href },
  };
  const expected = {
    uri: "file://$PROJECT/src/Page.vue",
    contents: `${project}/authored`,
    data: {
      fileName: "$VIRTUAL/src/pages/products/[id=int].vue.ts",
      name: "page",
      position: Buffer.byteLength(
        (`/// <reference path="$PROJECT/typed-router.d.ts" />\n` + body).slice(
          0,
          (`/// <reference path="$PROJECT/typed-router.d.ts" />\n` + body).lastIndexOf("page"),
        ),
      ),
    },
    revision: 3,
    extra: packet.extra,
  };
  assert.deepEqual(
    normalizeExperimentalRouterPackets(packet, [project], project, generated),
    expected,
  );
  for (const delta of [-1, 1])
    assert.throws(
      () =>
        assert.deepEqual(
          normalizeExperimentalRouterPackets(
            { ...packet, data: { ...packet.data, position: offset + delta } },
            [project],
            project,
            generated,
          ),
          expected,
        ),
      assert.AssertionError,
    );
  assert.throws(
    () =>
      normalizeExperimentalRouterPackets(
        { ...packet, data: { ...packet.data, fileName: "/foreign/file.ts" } },
        [project],
        project,
        generated,
      ),
    assert.AssertionError,
  );
});

test("frozen whole-document coordinates identify the independently authored query expression", async () => {
  const fs = await import("node:fs");
  const { createHash } = await import("node:crypto");
  for (const variant of ["lf", "crlf", "explicit"]) {
    const base = new URL(
      `../_fixtures/differential/lsp/experimental-router-params-10518/`,
      import.meta.url,
    );
    const expected = JSON.parse(fs.readFileSync(new URL(`${variant}.expected.json`, base), "utf8"));
    for (const kind of ["valid", "invalid"]) {
      const entry = expected.generated[kind];
      const bytes = fs.readFileSync(new URL(entry.file, base));
      assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.sha256);
      const code = bytes.toString("utf8");
      const prefix = "const page: number = route.params.";
      const offset = Buffer.byteLength(
        code.slice(0, code.indexOf(`${prefix}page`) + prefix.length),
      );
      assert.ok(offset >= prefix.length);
      assert.deepEqual(
        expected.requests[1].response.map(
          (item: { data: { vizeCompletion: { item: { data: { position: number } } } } }) =>
            item.data.vizeCompletion.item.data.position,
        ),
        [offset, offset, offset, offset, offset],
      );
      assert.deepEqual(
        expected.resolved.map(
          (item: { data: { vizeCompletion: { item: { data: { position: number } } } } }) =>
            item.data.vizeCompletion.item.data.position,
        ),
        [offset, offset, offset, offset, offset],
      );
    }
  }
});

test("fixed workspace roots are fresh Unicode children and reject infeasible owners", async () => {
  const fs = await import("node:fs");
  const os = await import("node:os");
  const path = await import("node:path");
  const owner = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "router-transport-")));
  try {
    const left = createExperimentalRouterWorkspace(owner, "left");
    const right = createExperimentalRouterWorkspace(owner, "right");
    assert.notEqual(left, right);
    assert.equal(left.length, 128);
    assert.equal(right.length, 128);
    assert.equal(path.dirname(left), owner);
    assert.equal(path.dirname(right), owner);
    assert.deepEqual(
      fs.readdirSync(owner).sort(),
      [path.basename(left), path.basename(right)].sort(),
    );
    const long = path.join(owner, "x".repeat(140));
    fs.mkdirSync(long);
    assert.throws(() => createExperimentalRouterWorkspace(long, "refused"), assert.AssertionError);
    assert.deepEqual(fs.readdirSync(long), []);
  } finally {
    fs.rmSync(owner, { recursive: true, force: true });
  }
});

test("different-length roots do not receive invented whole-code equivalence", () => {
  const original = `/// <reference path="${project}/typed-router.d.ts" />\n// @vize-map: 100:200 -> 0:10\n${body}`;
  const other = `${project}x`;
  const different = `/// <reference path="${other}/typed-router.d.ts" />\n// @vize-map: 101:201 -> 0:10\n${body}`;
  const expected = experimentalRouterDocumentTransport(original, project).normalizedSource;
  const actual = experimentalRouterDocumentTransport(different, other).normalizedSource;
  assert.throws(() => assert.equal(actual, expected), assert.AssertionError);
});

test("a proven apostrophe URI root accepts the native percent form without changing suffixes or text", () => {
  const quoted = "/tmp/q'雪😀";
  const native = pathToFileURL(quoted).href.replaceAll("'", "%27");
  const packet = {
    uri: `${native}/src/%5Bid%3Dint%5D.vue`,
    contents: `${native}/authored`,
    extra: { uri: `${native}-foreign/src/Page.vue` },
  };
  assert.deepEqual(normalizeExperimentalRouterPackets(packet, [quoted], quoted), {
    uri: "file://$PROJECT/src/%5Bid%3Dint%5D.vue",
    contents: packet.contents,
    extra: packet.extra,
  });
});

test("single-delimited compiler references preserve a literal double quote root", () => {
  const quoted = `${project}"`;
  const code = `/// <reference path='${quoted}/typed-router.d.ts' />\n${body}`;
  const expected = `/// <reference path='$PROJECT/typed-router.d.ts' />\n${body}`;
  const transport = experimentalRouterDocumentTransport(code, quoted);
  assert.equal(transport.normalizedSource, expected);
  const point = Buffer.byteLength(code.slice(0, code.lastIndexOf("page")));
  assert.equal(
    transport.relocate(point),
    Buffer.byteLength(expected.slice(0, expected.lastIndexOf("page"))),
  );
});

test("a delimiter embedded in a compiler reference value is rejected", () => {
  const both = `${project}'"`;
  for (const quote of ["'", '"']) {
    const broken = `/// <reference path=${quote}${both}/typed-router.d.ts${quote} />\n${body}`;
    assert.throws(() => experimentalRouterDocumentTransport(broken, both), assert.AssertionError);
  }
});
