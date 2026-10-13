import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { digest } from "./support/native-sdk-historical-packets.mjs";
import { validateLexicalDollarPackets } from "./support/native-sdk-historical-packets.mjs";
import { createPacketFixture, editJson, write } from "./support/native-lexical-dollar-fixture.mjs";

function fixtureTest(name, body) {
  void test(name, () => {
    const fixture = createPacketFixture();
    try {
      assert.doesNotThrow(() => validateLexicalDollarPackets(fixture.root));
      body(fixture);
    } finally {
      fixture.cleanup();
    }
  });
}
function refuses(fixture, file, mutation) {
  const original = fs.readFileSync(path.join(fixture.root, file));
  try {
    editJson(fixture.root, file, mutation);
    assert.throws(() => validateLexicalDollarPackets(fixture.root));
  } finally {
    write(fixture.root, file, original);
  }
}

fixtureTest(
  "complete nonexecuted packet fixture validates custody without runtime or performance credit",
  (fixture) => {
    assert.deepEqual(validateLexicalDollarPackets(fixture.root), {
      source: fixture.metadata.source,
      nativeCases: 9,
      producerLaws: 6,
      ignored: 0,
      apiLspCredit: false,
      original400Credit: false,
      performanceCredit: false,
    });
  },
);

fixtureTest(
  "missing, duplicate, foreign and incomplete executed cases refuse the full vector",
  (fixture) => {
    const file = "native/000-native-cli.json";
    const bytes = fs.readFileSync(path.join(fixture.root, file));
    fs.unlinkSync(path.join(fixture.root, file));
    assert.throws(() => validateLexicalDollarPackets(fixture.root));
    write(fixture.root, file, bytes);
    write(fixture.root, "native/009-native-cli.json", bytes);
    assert.throws(() => validateLexicalDollarPackets(fixture.root));
    fs.unlinkSync(path.join(fixture.root, "native/009-native-cli.json"));
    write(fixture.root, "native/000-same-owner.json", "{}\n");
    assert.throws(() => validateLexicalDollarPackets(fixture.root));
    fs.unlinkSync(path.join(fixture.root, "native/000-same-owner.json"));
    refuses(fixture, "native/normal-batch-results.json", (observations) => observations.pop());
    refuses(
      fixture,
      "native/normal-batch-results.json",
      (observations) => (observations[1] = observations[0]),
    );
    refuses(fixture, file, (packet) => (packet.completion = "PENDING"));
  },
);

fixtureTest(
  "skipped, missing or failed producer/native execution cannot earn acceptance",
  (fixture) => {
    for (const phase of ["producer", "native"]) {
      for (const replacement of [
        "test result: ok. 0 passed; 0 failed; 1 ignored;\n",
        "test result: ok. 0 passed; 0 failed; 0 ignored;\n",
      ]) {
        const file = `${phase}.stdout`;
        const original = fs.readFileSync(path.join(fixture.root, file));
        try {
          write(fixture.root, file, replacement);
          assert.throws(() => validateLexicalDollarPackets(fixture.root));
        } finally {
          write(fixture.root, file, original);
        }
      }
      const file = `${phase}.status`;
      write(fixture.root, file, "101\n");
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
      write(fixture.root, file, "0\n");
    }
    refuses(
      fixture,
      "producer/producer-scopes-true.json",
      (packet) => (packet.template = "foreign owner"),
    );
    refuses(
      fixture,
      "metadata.json",
      (metadata) => (metadata.runtimeEnvironment.requiredNative = "0"),
    );
  },
);

fixtureTest(
  "native identity and original completed native receipt must agree in whole bytes",
  (fixture) => {
    refuses(fixture, "metadata.json", (metadata) => (metadata.native.sha256 = "0".repeat(64)));
    refuses(fixture, "metadata.json", (metadata) => (metadata.native.versionProbe.status = 1));
    refuses(
      fixture,
      "metadata.json",
      (metadata) => (metadata.native.versionProbe.stderrBytes = [255]),
    );
    refuses(
      fixture,
      "native/normal-batch-results.json",
      (observations) => (observations[3].nativeBinary = "/foreign/native"),
    );
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.nativeSha256 = "f".repeat(64)),
    );
    const originalFile = "original-runtime/receipts.json";
    refuses(fixture, originalFile, (packet) => (packet.cases[0].nativeSha256 = "e".repeat(64)));
    refuses(
      fixture,
      "metadata.json",
      (metadata) => (metadata.compilerArtifact.features = ["default"]),
    );
    const observationsFile = "native/normal-batch-results.json";
    const observations = fs.readFileSync(path.join(fixture.root, observationsFile));
    const alias = path.join(fixture.root, "configured-native-alias");
    const foreign = path.join(fixture.root, "foreign-native");
    fs.symlinkSync(fixture.metadata.native.path, alias);
    try {
      editJson(fixture.root, observationsFile, (rows) => {
        for (const row of rows) row.nativeBinary = alias;
      });
      assert.doesNotThrow(() => validateLexicalDollarPackets(fixture.root));
      fs.unlinkSync(alias);
      fs.symlinkSync(path.join(fixture.root, "missing-native"), alias);
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
      fs.unlinkSync(alias);
      write(fixture.root, "foreign-native", "foreign executable bytes");
      fs.symlinkSync(foreign, alias);
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
      fs.copyFileSync(fixture.metadata.native.path, foreign);
      assert.deepEqual(fs.readFileSync(foreign), fs.readFileSync(fixture.metadata.native.path));
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
    } finally {
      write(fixture.root, observationsFile, observations);
    }
    const stdout = fs.readFileSync(path.join(fixture.root, "cargo.stdout"));
    try {
      write(
        fixture.root,
        "cargo.stdout",
        `${stdout.toString()}${JSON.stringify(fixture.metadata.compilerArtifact)}\n`,
      );
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
    } finally {
      write(fixture.root, "cargo.stdout", stdout);
    }
  },
);

fixtureTest(
  "self-consistent foreign original native authority still refuses the observed runtime",
  (fixture) => {
    const file = "original-runtime/receipts.json";
    const originalPath = fixture.metadata.originalReceipts[0].path;
    const oldReceipt = fs.readFileSync(path.join(fixture.root, file));
    const oldMetadata = fs.readFileSync(path.join(fixture.root, "metadata.json"));
    const value = JSON.parse(oldReceipt);
    value.cases[0].nativeSha256 = "e".repeat(64);
    const changed = Buffer.from(`${JSON.stringify(value)}\n`);
    try {
      write(fixture.root, file, changed);
      fs.writeFileSync(originalPath, changed);
      editJson(fixture.root, "metadata.json", (metadata) => {
        metadata.originalReceipts[0].sha256 = digest(changed);
        metadata.originalReceipts[0].byteLength = changed.length;
      });
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
    } finally {
      write(fixture.root, file, oldReceipt);
      fs.writeFileSync(originalPath, oldReceipt);
      write(fixture.root, "metadata.json", oldMetadata);
    }
  },
);

fixtureTest(
  "changed authored source, diagnostic vector and selected config refuse acceptance",
  (fixture) => {
    refuses(
      fixture,
      "native/normal-batch-results.json",
      (observations) => (observations[0].inputs[1].source += "\n"),
    );
    refuses(
      fixture,
      "native/normal-batch-results.json",
      (observations) => observations[1].diagnostics[0].column--,
    );
    refuses(
      fixture,
      "native/normal-batch-results.json",
      (observations) => (observations[1].diagnostics = []),
    );
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.projectRoot = "/foreign/project"),
    );
    refuses(fixture, "native/000-native-cli.json", (packet) => (packet.actualCwd = "/foreign/cwd"));
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.authoredConfigBytes = "{}\n"),
    );
    refuses(fixture, "native/000-native-cli.json", (packet) => {
      const config = JSON.parse(packet.requestedConfigBytes);
      config.compilerOptions.strict = false;
      packet.requestedConfigBytes = JSON.stringify(config);
    });
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.actualArgs[5] = "/foreign/tsconfig.json"),
    );
  },
);

fixtureTest(
  "partial generated sources/maps and inconsistent terminal raw streams refuse acceptance",
  (fixture) => {
    refuses(fixture, "native/000-native-cli.json", (packet) => packet.generatedFiles.pop());
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.generatedFiles[0].code = ""),
    );
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.generatedFiles[0].sourceMap = "CompositeSourceMap {\n}"),
    );
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) =>
        (packet.generatedFiles[0].sourceMap =
          "CompositeSourceMap {\n [gen_range: 0..1, src_range: 0..1,\n}"),
    );
    refuses(fixture, "native/000-native-cli.json", (packet) => {
      packet.generatedFiles[0].sourceMap = packet.generatedFiles[0].sourceMap.replace(
        " import_map: ImportSourceMap { adjustments: [], },\n",
        "",
      );
    });
    refuses(
      fixture,
      "native/000-native-cli.json",
      (packet) => (packet.generatedFiles[0].authoredSource += "\n"),
    );
    refuses(fixture, "native/000-native-cli.json", (packet) => (packet.rawStdoutBytes = [255]));
    refuses(fixture, "native/001-native-cli.json", (packet) => (packet.rawStdoutBytes = []));
    refuses(fixture, "native/001-native-cli.json", (packet) => (packet.rawStderrBytes = [255]));
    refuses(fixture, "native/001-native-cli.json", (packet) => (packet.status = "exit status: 0"));
  },
);

fixtureTest(
  "source HEAD, whole copied bodies and eight-input custody remain immutable",
  (fixture) => {
    refuses(fixture, "metadata.json", (metadata) => (metadata.source.head = "0".repeat(40)));
    refuses(fixture, "metadata.json", (metadata) => metadata.copied.pop());
    refuses(fixture, "metadata.json", (metadata) => (metadata.copied[0].gitEntryHex = ""));
    const file = "corpus/valid.vue.txt";
    const bytes = fs.readFileSync(path.join(fixture.root, file));
    try {
      write(fixture.root, file, Buffer.concat([bytes, Buffer.from("\n")]));
      assert.throws(() => validateLexicalDollarPackets(fixture.root));
    } finally {
      write(fixture.root, file, bytes);
    }
    refuses(fixture, "corpus/expected.json", (vector) => (vector[1].diagnostics = []));
  },
);
