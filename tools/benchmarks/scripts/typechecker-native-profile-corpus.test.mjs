/** Real duplicate-tree orchestration with controlled vectors, never native timing evidence. */
import assert from "node:assert/strict";
import { readFileSync, rmSync, statSync, utimesSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import test from "node:test";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";
import { profileDuplicateCorpus } from "./typechecker-native-profile-corpus.mjs";
import { editArchive, fixture } from "./typechecker-native-profile-corpus-fixture.mjs";

const graphError = /config|member|graph|context/u;
function cleanup(context) {
  rmSync(context.root, { recursive: true, force: true });
}

await test("fresh duplicate preserves authored bytes, symlink and exact orchestration", () => {
  const context = fixture();
  try {
    const receipt = profileDuplicateCorpus(context.options);
    assert.deepEqual(context.calls, [
      `${context.label}-profile-before`,
      `${context.label}-profile`,
      `${context.label}-profile-after`,
    ]);
    assert.deepEqual(corpusManifest(context.duplicateRoot), corpusManifest(context.original));
    assert.deepEqual(receipt.inputManifest, context.options.corpus.manifest);
    assert.equal(receipt.originalRoot, context.original);
    assert.equal(receipt.duplicateRoot, context.duplicateRoot);
    assert.equal(receipt.directReceipt, "controlled-direct");
    assert.equal(receipt.wrappedReceipt, "controlled-wrapped");
    assert.deepEqual(
      receipt.profiles.map(({ config }) => basename(config)),
      ["tsconfig.shard0.json", "tsconfig.shard1.json"],
    );
    assert.equal(receipt.projectionBeforeSha256, receipt.projectionAfterSha256);
    for (const key of [
      "orderedProductReportEqual",
      "virtualBytesEqual",
      "orderedNativeGraphBytesEqual",
      "canonicalExternalContextEqual",
      "excludedFromTimings",
    ])
      assert.equal(receipt[key], true);
    assert.equal(
      receipt.semanticProfileDecoding,
      "not implemented; gzip container validation only",
    );
    for (const key of [
      "cpuSamples",
      "allocationSamples",
      "phaseAttribution",
      "startupMs",
      "programConstructionMs",
    ])
      assert.equal(receipt[key], null);
    assert.equal(Object.hasOwn(receipt, "speedup"), false);
  } finally {
    cleanup(context);
  }
});

const guardCases = [
  [
    "nested diagnostic message",
    ({ report }) => {
      report.files[0].diagnostics[0].relatedInformation[0].message = "changed";
    },
    /full ordered product report/u,
  ],
  [
    "diagnostic file order",
    ({ report }) => {
      report.files.reverse();
    },
    /full ordered product report/u,
  ],
  [
    "diagnostic coordinates",
    ({ report }) => {
      report.files[0].diagnostics[0].range.end.character++;
    },
    /full ordered product report/u,
  ],
  [
    "virtual hash",
    ({ checked }) => {
      checked.direct.virtualFiles[0].sha256 = "changed";
    },
    /virtual bytes/u,
  ],
  [
    "virtual order",
    ({ checked }) => {
      checked.direct.virtualFiles.reverse();
    },
    /virtual bytes/u,
  ],
  [
    "config hash",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].configSha256 = "changed";
    },
    graphError,
  ],
  [
    "compiler options",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].compilerOptions.types = ["other-global"];
    },
    graphError,
  ],
  [
    "checker argument",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].args[1] = "2";
    },
    graphError,
  ],
  [
    "shard order",
    ({ checked }) => {
      checked.wrapped.nativeShards.reverse();
    },
    graphError,
  ],
  [
    "member order",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].members.reverse();
    },
    graphError,
  ],
  [
    "member omission",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].members.pop();
    },
    graphError,
  ],
  [
    "member bytes",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].members[0].bytes++;
    },
    graphError,
  ],
  [
    "member hash",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].members[0].sha256 = "changed";
    },
    graphError,
  ],
  [
    "graph pointer hash",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].nativeGraphArchive.sha256 = "changed";
    },
    graphError,
  ],
  [
    "missing profile",
    ({ checked }) => {
      delete checked.wrapped.nativeShards[0].nativeProfile;
    },
    /missing successful native CPU replay/u,
  ],
  [
    "failed profile",
    ({ checked }) => {
      checked.wrapped.nativeShards[0].nativeProfile.validation = "failed";
    },
    /missing successful native CPU replay/u,
  ],
  [
    "zero native commands",
    ({ checked, reference }) => {
      checked.wrapped.nativeShards = [];
      reference.wrapped.nativeShards = [];
    },
    /no native commands/u,
  ],
];
for (const [name, mutate, message] of guardCases) {
  await test(`fail closed on ${String(name)}`, () => {
    const context = fixture((values) => {
      if (values.stage === "pair") mutate(values);
    });
    try {
      assert.throws(() => profileDuplicateCorpus(context.options), message);
      assert.deepEqual(context.calls, [
        `${context.label}-profile-before`,
        `${context.label}-profile`,
      ]);
    } finally {
      cleanup(context);
    }
  });
}

for (const [name, change] of [
  [
    "external realpath",
    (member) => {
      member.realPath += "-other-library";
    },
  ],
  [
    "external file revision",
    (member) => {
      member.revision.ino = "other-inode";
    },
  ],
  [
    "external symlink target",
    (member) => {
      member.links[0].target += "-retargeted";
    },
  ],
]) {
  await test(`valid archived bytes cannot hide changed ${String(name)}`, () => {
    const context = fixture(({ checked, stage }) => {
      if (stage !== "pair") return;
      editArchive(checked.wrapped.nativeShards[0], (graph) => {
        for (const members of [graph.membersBefore, graph.membersAfter]) change(members.at(-1));
      });
    });
    try {
      assert.throws(() => profileDuplicateCorpus(context.options), /canonical external context/u);
    } finally {
      cleanup(context);
    }
  });
}

await test("correct cache keys under different native cache parents cannot relocate", () => {
  const context = fixture(({ context, stage }) => {
    if (stage === "before") context.alternateCache = true;
  });
  try {
    assert.throws(() => profileDuplicateCorpus(context.options), /canonical external context/u);
  } finally {
    cleanup(context);
  }
});

await test("generated projection code, maps and links must remain complete and unchanged", () => {
  for (const change of [
    (projection) => {
      projection.files[0].code += "changed";
    },
    (projection) => {
      projection.files[0].maps[0].generated[1]++;
    },
    (projection) => {
      projection.files[0].links.pop();
    },
  ]) {
    const context = fixture(({ projection, stage }) => {
      if (stage === "after") change(projection);
    });
    try {
      assert.throws(
        () => profileDuplicateCorpus(context.options),
        /changed generated code\/maps\/links/u,
      );
    } finally {
      cleanup(context);
    }
  }
});

for (const target of ["original", "duplicateRoot"]) {
  await test(`same-length restored-mtime ${target} source change cannot pass`, () => {
    const context = fixture(({ context, stage }) => {
      if (stage !== "pair") return;
      const file = join(context[target], "B.vue");
      const before = statSync(file);
      const bytes = readFileSync(file);
      const changed = Buffer.from(bytes);
      changed[changed.indexOf("absent")] = "p".charCodeAt(0);
      writeFileSync(file, changed);
      utimesSync(file, before.atime, before.mtime);
      assert.equal(readFileSync(file).length, bytes.length);
    });
    try {
      assert.throws(
        () => profileDuplicateCorpus(context.options),
        target === "original" ? /changed original corpus/u : /changed duplicate corpus/u,
      );
    } finally {
      cleanup(context);
    }
  });
}

await test("reused profile corpus root is rejected before any additional callbacks", () => {
  const context = fixture();
  try {
    profileDuplicateCorpus(context.options);
    const calls = [...context.calls];
    const manifest = corpusManifest(context.duplicateRoot);
    assert.throws(() => profileDuplicateCorpus(context.options), /fresh duplicate tree/u);
    assert.deepEqual(context.calls, calls);
    assert.deepEqual(corpusManifest(context.duplicateRoot), manifest);
  } finally {
    cleanup(context);
  }
});
