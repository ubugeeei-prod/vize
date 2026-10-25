import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  BUILD_INFO_SOURCE,
  buildInfoTopologyFingerprint,
  canonicalBuildInfoArgument,
} from "./typechecker-native-build-info.mjs";
import { INFRASTRUCTURE_PATHS } from "./typechecker-native-source-custody.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const write = (file, bytes) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, bytes);
};
const namespace = ["compiler-sfc", "compiler-vapor"];

// Independent source-recipe input construction; the production helper is not
// used to manufacture its own expected physical filenames.
function expectedFingerprint(links) {
  const bytes = [Buffer.from("vize-cli-build-info-package-topology:v1\0")];
  for (const { path, target } of links) {
    bytes.push(Buffer.from("link\0"));
    for (const text of [path, target]) {
      const value = Buffer.from(text);
      const size = Buffer.alloc(8);
      size.writeBigUInt64LE(BigInt(value.length));
      bytes.push(size, value);
    }
  }
  return sha(Buffer.concat(bytes));
}

function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "native-buildinfo-")));
  const sourceRoot = path.join(root, "source");
  write(
    path.join(sourceRoot, BUILD_INFO_SOURCE.path),
    fs.readFileSync(path.join(ROOT, BUILD_INFO_SOURCE.path)),
  );
  const modules = path.join(root, "driver", "tools", "benchmarks", "scripts", "node_modules");
  fs.mkdirSync(path.join(modules, "@vue"), { recursive: true });
  const selectedVuePackageDir = path.join(modules, "vue");
  const vue = path.join(root, "external", "vue");
  write(path.join(vue, "package.json"), '{"name":"vue","version":"3.6.0-beta.10"}');
  fs.symlinkSync(vue, selectedVuePackageDir, "dir");
  for (const name of namespace) {
    const target = path.join(root, "external", name);
    write(
      path.join(target, "package.json"),
      JSON.stringify({ name: "@vue/" + name, version: "3.6.0-beta.10" }),
    );
    fs.symlinkSync(target, path.join(modules, "@vue", name), "dir");
  }
  function project(name) {
    const authoredRoot = path.join(root, name);
    fs.mkdirSync(path.join(authoredRoot, "node_modules"), { recursive: true });
    fs.symlinkSync(selectedVuePackageDir, path.join(authoredRoot, "node_modules", "vue"), "dir");
    fs.symlinkSync(
      path.join(modules, "@vue"),
      path.join(authoredRoot, "node_modules", "@vue"),
      "dir",
    );
    write(path.join(authoredRoot, "package.json"), '{"private":true,"type":"module"}');
    const owner = sha(
      Buffer.concat([Buffer.from("vize-canon-project-key:v1\0"), Buffer.from(authoredRoot)]),
    );
    const cwd = path.join(root, "cache", "projects", owner);
    const links = namespace.map((name) => ({
      path: path.join(cwd, "node_modules", "@vue", name),
      target: fs.realpathSync(path.join(modules, "@vue", name)),
    }));
    for (const link of links) {
      fs.mkdirSync(path.dirname(link.path), { recursive: true });
      fs.symlinkSync(link.target, link.path, "dir");
    }
    const receipt = path.join(cwd, ".vize-buildinfo", "package-links.json");
    write(receipt, JSON.stringify({ version: 1, links }));
    const config = path.join(cwd, "tsconfig.json");
    write(config, "{}");
    const value = path.join(
      cwd,
      ".vize-buildinfo",
      "tsconfig." + expectedFingerprint(links) + ".tsbuildinfo",
    );
    const shard = {
      cwd,
      config,
      args: [
        "--checkers",
        "1",
        "--incremental",
        "--tsBuildInfoFile",
        value,
        "--pretty",
        "false",
        "--project",
        config,
      ],
    };
    const options = {
      shard,
      authoredRoot,
      corpusId: "shared-barrel500",
      selectedVuePackageDir,
      sourceRoot,
    };
    return { options, shard, authoredRoot, receipt, links };
  }
  return {
    root,
    modules,
    sourceRoot,
    project,
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

await test("captured original and duplicate filenames reproduce exact source-derived digests", () => {
  const records = [
    [
      "488e9fdda98db94a973cb6eb15bc7156e0733a10ee66598118c927adf92be1d9",
      "0bbded0aba56ac02c2c960f27d9003a9aecb9ad43791066eb657bec98e5584cd",
    ],
    [
      "8d7e734309a65523562cd9c92cfe0b60a6bc4f28cbc3bd65537fddf1125b7b1d",
      "229488e02dc0a93720520d1464b06d09de60842ad7e98c4ba624c6352169e880",
    ],
  ];
  for (const [owner, expected] of records) {
    const links = namespace.map((name) => ({
      path: `/home/runner/.cache/vize/canon/projects/${owner}/node_modules/@vue/${name}`,
      target: `/home/runner/_work/vize/vize/node_modules/.pnpm/@vue+${name}@3.6.0-beta.10/node_modules/@vue/${name}`,
    }));
    assert.equal(buildInfoTopologyFingerprint(links), expected);
  }
  assert(INFRASTRUCTURE_PATHS.has("tools/benchmarks/scripts/typechecker-native-build-info.mjs"));
  assert(
    INFRASTRUCTURE_PATHS.has("tools/benchmarks/scripts/typechecker-native-build-info.test.mjs"),
  );
});

await test("only a proven owner-dependent value moves, with literal external targets", () => {
  const context = fixture();
  try {
    const first = context.project("original 雪🙂");
    const second = context.project("duplicate 雪🙂");
    const raw = structuredClone([first.shard.args, second.shard.args]);
    assert.notEqual(first.shard.args[4], second.shard.args[4]);
    const result = canonicalBuildInfoArgument(first.options);
    assert.equal(result, canonicalBuildInfoArgument(second.options));
    assert.match(result, /^<virtual>\/\.vize-buildinfo\/tsconfig\.[0-9a-f]{64}\.tsbuildinfo$/u);
    assert.deepEqual([first.shard.args, second.shard.args], raw, "raw argv remains immutable");
    assert.deepEqual(
      fs.readdirSync(path.dirname(first.receipt)),
      ["package-links.json"],
      "observer writes no sidecar or cache state",
    );
    const changed = first.links.map((link) => ({
      ...link,
      target: link.target + "-same-bytes-other-context",
    }));
    assert.notEqual(
      buildInfoTopologyFingerprint(changed),
      buildInfoTopologyFingerprint(first.links),
    );
  } finally {
    context.cleanup();
  }
});

await test("same-byte packages at different canonical external contexts stay distinct", () => {
  const first = fixture();
  const second = fixture();
  try {
    const a = first.project("original");
    const b = second.project("duplicate");
    assert.deepEqual(
      fs.readFileSync(path.join(a.links[0].target, "package.json")),
      fs.readFileSync(path.join(b.links[0].target, "package.json")),
    );
    assert.notEqual(canonicalBuildInfoArgument(a.options), canonicalBuildInfoArgument(b.options));
  } finally {
    first.cleanup();
    second.cleanup();
  }
});

const negatives = [
  [
    "digest nibble",
    (c) => {
      c.shard.args[4] = c.shard.args[4].replace(/([0-9a-f])(?=\.tsbuildinfo$)/u, (x) =>
        x === "0" ? "1" : "0",
      );
    },
  ],
  [
    "filename stem",
    (c) => {
      c.shard.args[4] = c.shard.args[4].replace("/tsconfig.", "/other.");
    },
  ],
  [
    "filename extension",
    (c) => {
      c.shard.args[4] += ".extra";
    },
  ],
  [
    "foreign root",
    (c) => {
      c.shard.args[4] = c.shard.args[4].replace(c.shard.cwd, c.shard.cwd + "-other");
    },
  ],
  [
    "foreign cache directory",
    (c) => {
      c.shard.args[4] = c.shard.args[4].replace("/.vize-buildinfo/", "/.other-cache/");
    },
  ],
  [
    "flag at minus one",
    (c) => {
      [c.shard.args[2], c.shard.args[3]] = [c.shard.args[3], c.shard.args[2]];
    },
  ],
  [
    "flag at plus one",
    (c) => {
      [c.shard.args[3], c.shard.args[4]] = [c.shard.args[4], c.shard.args[3]];
    },
  ],
  [
    "missing flag",
    (c) => {
      c.shard.args.splice(3, 2);
    },
  ],
  [
    "duplicate flag",
    (c) => {
      c.shard.args.push("--tsBuildInfoFile", c.shard.args[4]);
    },
  ],
  [
    "adjacent flag",
    (c) => {
      c.shard.args[5] = "--other";
    },
  ],
  [
    "config argument",
    (c) => {
      c.shard.args[8] += ".other";
    },
  ],
  [
    "config owner",
    (c) => {
      c.shard.config = path.join(c.authoredRoot, "tsconfig.json");
      c.shard.args[8] = c.shard.config;
    },
  ],
  [
    "virtual owner",
    (c) => {
      c.options.authoredRoot += "-other";
    },
  ],
  [
    "unsupported corpus",
    (c) => {
      c.options.corpusId = "controlled";
    },
  ],
  [
    "unknown source",
    (c, context) => {
      fs.appendFileSync(path.join(context.sourceRoot, BUILD_INFO_SOURCE.path), "\n// changed\n");
    },
  ],
  [
    "unknown receipt version",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.version++;
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "extra receipt key",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.extra = true;
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "missing receipt link",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.links.pop();
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "extra receipt link",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.links.push(r.links[0]);
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "reordered receipt",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.links.reverse();
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "retargeted receipt",
    (c) => {
      const r = JSON.parse(fs.readFileSync(c.receipt));
      r.links[0].target += "-other";
      fs.writeFileSync(c.receipt, JSON.stringify(r));
    },
  ],
  [
    "extra root package",
    (c) => {
      fs.mkdirSync(path.join(c.authoredRoot, "node_modules", "extra"));
    },
  ],
  [
    "extra nested install",
    (c) => {
      fs.mkdirSync(path.join(c.authoredRoot, "nested", "node_modules"), { recursive: true });
    },
  ],
  [
    "extra ancestor install",
    (c, context) => {
      fs.mkdirSync(path.join(context.root, "node_modules"));
    },
  ],
  [
    "extra namespace",
    (c, context) => {
      fs.mkdirSync(path.join(context.modules, "@vue", "extra"));
    },
  ],
  [
    "missing namespace",
    (c, context) => {
      fs.unlinkSync(path.join(context.modules, "@vue", "compiler-sfc"));
    },
  ],
  [
    "changed dependency identity",
    (c) => {
      write(
        path.join(c.links[0].target, "package.json"),
        '{"name":"@vue/other","version":"3.6.0-beta.10"}',
      );
    },
  ],
  [
    "changed dependency version",
    (c) => {
      write(
        path.join(c.links[0].target, "package.json"),
        '{"name":"@vue/compiler-sfc","version":"other"}',
      );
    },
  ],
  [
    "source dependency route",
    (c) => {
      const p = path.join(c.authoredRoot, "node_modules", "vue");
      const t = fs.readlinkSync(p);
      fs.unlinkSync(p);
      fs.symlinkSync(t + "/.", p, "dir");
    },
  ],
  [
    "retargeted same-byte link",
    (c, context) => {
      const p = c.links[0].path;
      const t = path.join(context.root, "same-byte-target");
      fs.cpSync(c.links[0].target, t, { recursive: true });
      fs.unlinkSync(p);
      fs.symlinkSync(t, p, "dir");
    },
  ],
  [
    "symlink receipt",
    (c) => {
      const bytes = fs.readFileSync(c.receipt);
      fs.unlinkSync(c.receipt);
      write(c.receipt + ".other", bytes);
      fs.symlinkSync(c.receipt + ".other", c.receipt);
    },
  ],
];
for (const [name, mutate] of negatives)
  await test("build-info relocation refuses " + name, () => {
    const context = fixture();
    try {
      const c = context.project("original");
      mutate(c, context);
      assert.throws(() => canonicalBuildInfoArgument(c.options));
    } finally {
      context.cleanup();
    }
  });

await test("controlled no-buildinfo argv is literal and needs no topology exception", () => {
  const shard = { args: ["--project", "/controlled/tsconfig.json"] };
  const original = structuredClone(shard);
  assert.equal(canonicalBuildInfoArgument({ shard, corpusId: "controlled" }), null);
  assert.deepEqual(shard, original);
});
