import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { prepareVueFixture } from "./prepare-vue.ts";
import { loadVueFixtureAuthority } from "./vue.ts";

const sourceHead = "c2bc3a55a2b70dc4dd6c4e43ee0737c347476398";
const source = {
  head: sourceHead,
  files: {
    "pnpm-lock.yaml": "6c311bea2e8c89e938f1fe4f4c3c87e08d5776022a316070538178e61ef2ea86",
    "playground/package.json": "67a5b2a66ab71004c6440696ae5c1a3824705c6a629433afd5ad462d44abda6e",
    "crates/vize/tests/support/lsp_vue_project.rs":
      "352aa0542cbfa01407eded744e207957484fd5ab0c0a4a9736f71c8dd869f168",
  },
};
function rejected(
  receipt: unknown,
  overrides: Partial<Parameters<typeof loadVueFixtureAuthority>[0]> = {},
) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vue-custody-law-"));
  try {
    const bytes = `${JSON.stringify(receipt)}\n`;
    const receiptPath = path.join(root, "receipt.json");
    fs.writeFileSync(receiptPath, bytes);
    assert.throws(() =>
      loadVueFixtureAuthority({
        receiptPath,
        receiptSha256: createHash("sha256").update(bytes).digest("hex"),
        sourceRoot: fs.realpathSync(root),
        sourceHead,
        ...overrides,
      }),
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
}
const receipt = () => ({
  schema: "vize-stock-alias-vue-install-v1",
  source: structuredClone(source),
});

test("an independently supplied receipt digest cannot be substituted", () => {
  rejected(receipt(), { receiptSha256: "0".repeat(64) });
});
test("a runtime package receipt is not stock Vue fixture authority", () => {
  rejected({ ...receipt(), schema: "vize-public-registry-install-v1" });
});
test("a newer repository head cannot redefine the original stock dependency", () => {
  const different = "1".repeat(40);
  rejected({ ...receipt(), source: { ...source, head: different } }, { sourceHead: different });
});
test("receipt source head must equal the caller's original stock head", () => {
  rejected({ ...receipt(), source: { ...source, head: "2".repeat(40) } });
});
test("stable Vue or an ambient manifest cannot replace the original Playground bytes", () => {
  const wrong = receipt();
  wrong.source.files["playground/package.json"] = "3".repeat(64);
  rejected(wrong);
});
test("all original lock and helper files must be authenticated", () => {
  for (const key of ["pnpm-lock.yaml", "crates/vize/tests/support/lsp_vue_project.rs"] as const) {
    const wrong = receipt();
    wrong.source.files[key] = "4".repeat(64);
    rejected(wrong);
  }
});
test("a claimed source receipt without the actual historical git files fails closed", () => {
  rejected(receipt());
});

async function preparationLaw(run: (options: Parameters<typeof prepareVueFixture>[0]) => void) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vue-prepare-law-")));
  const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
  const npmPath = path.join(root, "npm-cli.js");
  // Execution of this input would be a test failure; every law must stop before installation.
  fs.writeFileSync(npmPath, `throw new Error("preflight must reject before npm executes");\n`);
  const nodePath = fs.realpathSync(process.execPath);
  try {
    run({
      nodePath,
      nodeSha256: digest(fs.readFileSync(nodePath)),
      npmPath,
      npmSha256: digest(fs.readFileSync(npmPath)),
      sourceRoot: root,
      outputRoot: path.join(root, "fresh"),
    });
  } finally {
    fs.rmSync(root, { recursive: true });
  }
}
test("collector never overwrites an existing fixture install", async () =>
  preparationLaw((options) => {
    assert.throws(
      () => prepareVueFixture({ ...options, outputRoot: options.sourceRoot }),
      /fresh fixture output required/u,
    );
  }));
test("collector authenticates Node bytes before creating an install directory", async () =>
  preparationLaw((options) => {
    assert.throws(() => prepareVueFixture({ ...options, nodeSha256: "0".repeat(64) }));
    assert.equal(fs.existsSync(options.outputRoot), false);
  }));
test("collector authenticates npm bytes before creating an install directory", async () =>
  preparationLaw((options) => {
    assert.throws(() => prepareVueFixture({ ...options, npmSha256: "0".repeat(64) }));
    assert.equal(fs.existsSync(options.outputRoot), false);
  }));
test("collector cannot install without the actual original stock git source", async () =>
  preparationLaw((options) => {
    assert.throws(() => prepareVueFixture(options));
    assert.equal(fs.existsSync(options.outputRoot), false);
  }));
