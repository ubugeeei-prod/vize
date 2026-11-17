import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  evaluateGlibcFloor,
  parseGlibcVersion,
  verifyGlibcSymbols,
} from "../../tools/commands/ci/github/verify-glibc-symbols.ts";
import { readElfDynamicRequirements } from "../../tools/support/github/elf-dynamic-requirements.ts";
import { glibcFloorTarget, linuxGnuGlibcFloor } from "../../tools/support/github/glibc-floor.ts";
import { nativeReleasePlatforms } from "../../tools/support/github/release-platforms.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";
import { syntheticElf } from "./support/synthetic-elf.ts";

const floor = parseGlibcVersion(linuxGnuGlibcFloor);

// The shape of the published 0.777.0 bindings, which Amazon Linux 2023 (glibc 2.34) rejects.
const raisedFloorImage = syntheticElf({
  needed: ["libgcc_s.so.1", "libm.so.6", "libc.so.6", "ld-linux-x86-64.so.2"],
  requirements: [
    { library: "libm.so.6", versions: ["GLIBC_2.2.5", "GLIBC_2.35"] },
    { library: "libc.so.6", versions: ["GLIBC_2.2.5", "GLIBC_2.34"] },
    { library: "libgcc_s.so.1", versions: ["GCC_3.0"] },
  ],
  imports: [
    { symbol: "hypotf", version: "GLIBC_2.35" },
    { symbol: "pthread_create", version: "GLIBC_2.34" },
    { symbol: "dlsym", version: "GLIBC_2.34" },
    { symbol: "memcpy", version: "GLIBC_2.2.5" },
    { symbol: "_Unwind_Resume", version: "GCC_3.0" },
  ],
});

const floorImage = syntheticElf({
  machine: 183,
  needed: ["libm.so.6", "libpthread.so.0", "libc.so.6", "libdl.so.2"],
  requirements: [
    { library: "libc.so.6", versions: ["GLIBC_2.17"] },
    { library: "libm.so.6", versions: ["GLIBC_2.17"] },
  ],
  imports: [
    { symbol: "memcpy", version: "GLIBC_2.17" },
    { symbol: "expf", version: "GLIBC_2.17" },
  ],
});

test("the ELF reader maps every imported symbol to the library version it requires", () => {
  const image = readElfDynamicRequirements(raisedFloorImage);
  assert.equal(image.machine, "x86_64");
  assert.equal(image.packedRelativeRelocations, false);
  assert.deepEqual(image.needed, [
    "libgcc_s.so.1",
    "libm.so.6",
    "libc.so.6",
    "ld-linux-x86-64.so.2",
  ]);
  assert.deepEqual(
    image.requirements.map(({ library, version, symbols }) => [library, version, symbols]),
    [
      ["libc.so.6", "GLIBC_2.2.5", ["memcpy"]],
      ["libc.so.6", "GLIBC_2.34", ["dlsym", "pthread_create"]],
      ["libgcc_s.so.1", "GCC_3.0", ["_Unwind_Resume"]],
      ["libm.so.6", "GLIBC_2.2.5", []],
      ["libm.so.6", "GLIBC_2.35", ["hypotf"]],
    ],
  );
});

test("the floor gate names each symbol that needs a newer glibc", () => {
  const report = evaluateGlibcFloor(readElfDynamicRequirements(raisedFloorImage), floor);
  assert.equal(report.highest?.text, "2.35");
  assert.deepEqual(report.violations, [
    "requires GLIBC_2.34 from libc.so.6 (dlsym, pthread_create), above the glibc 2.17 floor",
    "requires GLIBC_2.35 from libm.so.6 (hypotf), above the glibc 2.17 floor",
  ]);
});

test("the floor gate accepts an image linked against the floor", () => {
  const image = readElfDynamicRequirements(floorImage);
  assert.equal(image.machine, "aarch64");
  assert.deepEqual(evaluateGlibcFloor(image, floor), { highest: floor, violations: [] });
});

test("the floor gate rejects DT_RELR, private versions, libgcc_s past GCC 4.8, and foreign libraries", () => {
  const image = readElfDynamicRequirements(
    syntheticElf({
      needed: ["libc.so.6", "libgcc_s.so.1", "libstdc++.so.6"],
      requirements: [
        { library: "libc.so.6", versions: ["GLIBC_2.17", "GLIBC_ABI_DT_RELR", "GLIBC_PRIVATE"] },
        { library: "libgcc_s.so.1", versions: ["GCC_4.2.0", "GCC_12.0.0"] },
      ],
      imports: [
        { symbol: "_dl_find_object", version: "GLIBC_PRIVATE" },
        { symbol: "__aarch64_cas8_acq_rel", version: "GCC_12.0.0" },
      ],
      relr: true,
    }),
  );
  assert.equal(image.packedRelativeRelocations, true);
  assert.deepEqual(evaluateGlibcFloor(image, floor).violations, [
    "links libstdc++.so.6, which is not part of the base glibc runtime",
    "requires GLIBC_ABI_DT_RELR (packed relative relocations), which only glibc 2.36+ provides",
    "requires GLIBC_PRIVATE from libc.so.6 (_dl_find_object)",
    "requires GCC_12.0.0 from libgcc_s.so.1 (__aarch64_cas8_acq_rel), newer than the GCC 4.8.0 runtime of glibc 2.17 systems",
  ]);
  const relrOnly = readElfDynamicRequirements(
    syntheticElf({
      needed: ["libc.so.6"],
      requirements: [{ library: "libc.so.6", versions: ["GLIBC_2.17"] }],
      imports: [],
      relr: true,
    }),
  );
  assert.deepEqual(evaluateGlibcFloor(relrOnly, floor).violations, [
    "uses packed relative relocations (DT_RELR), which only glibc 2.36+ loads",
  ]);
});

test("the floor gate command fails on raised images and passes floor images", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glibc-floor-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const raised = path.join(directory, "raised.node");
  const atFloor = path.join(directory, "floor.node");
  const notElf = path.join(directory, "script.node");
  fs.writeFileSync(raised, raisedFloorImage);
  fs.writeFileSync(atFloor, floorImage);
  fs.writeFileSync(notElf, "#!/bin/sh\n");
  const errors: string[] = [];
  const logs: string[] = [];
  t.mock.method(console, "error", (line: string) => errors.push(line));
  t.mock.method(console, "log", (line: string) => logs.push(line));

  assert.equal(verifyGlibcSymbols([atFloor]), 0);
  assert.deepEqual(logs, [
    `${atFloor}: aarch64, GLIBC_2.17 <= glibc 2.17 floor; needs libm.so.6, libpthread.so.0, libc.so.6, libdl.so.2`,
  ]);
  assert.throws(
    () => verifyGlibcSymbols([raised, notElf]),
    /glibc 2\.17 floor verification failed/,
  );
  assert.deepEqual(errors, [
    `::error::${raised} requires GLIBC_2.34 from libc.so.6 (dlsym, pthread_create), above the glibc 2.17 floor`,
    `::error::${raised} requires GLIBC_2.35 from libm.so.6 (hypotf), above the glibc 2.17 floor`,
    `::error::${notElf}: not an ELF file`,
  ]);
  assert.equal(verifyGlibcSymbols(["--max", "2.35", raised]), 0);
});

test("release builds link every GNU artifact against the documented floor", () => {
  assert.equal(linuxGnuGlibcFloor, "2.17");
  assert.equal(glibcFloorTarget("x86_64-unknown-linux-gnu"), "x86_64-unknown-linux-gnu.2.17");
  assert.equal(glibcFloorTarget("aarch64-unknown-linux-gnu"), "aarch64-unknown-linux-gnu.2.17");
  assert.equal(glibcFloorTarget("x86_64-unknown-linux-musl"), "x86_64-unknown-linux-musl");
  for (const platform of nativeReleasePlatforms.filter(({ target }) => target.endsWith("-gnu"))) {
    assert.equal(platform.cross_compile, true, `${platform.target} must link through zigbuild`);
  }
  // The loader's diagnostics quote the same floor.
  assert.match(
    readRepoFile("npm", "native", "native-binding.js"),
    new RegExp(`const linuxGnuGlibcFloor = "${linuxGnuGlibcFloor.replace(".", "\\.")}";`),
  );

  const native = readRepoFile(".github", "actions", "build-release-native", "action.yml");
  assert.match(
    native,
    /verify-glibc-symbols\.ts npm\/native\/\*\.linux-\*-gnu\.node[\s\S]*verify-glibc-symbols\.ts npm\/fresco-native\/\*\.linux-\*-gnu\.node/,
  );
  assert.doesNotMatch(native, /verify-glibc-symbols\.ts --max/);
  assert.match(native, /uses: \.\/\.github\/actions\/setup-cargo-zigbuild/);

  const cli = readRepoFile(".github", "actions", "build-release-cli", "action.yml");
  assert.match(cli, /glibc-floor\.ts target "\$RELEASE_TARGET"/);
  assert.match(cli, /cargo zigbuild --locked --release -p vize --target "\$floor_target"/);
  assert.match(cli, /verify-glibc-symbols\.ts "target\/\$RELEASE_TARGET\/release\/vize"/);

  const job = workflowJobBody(readRepoFile(".github", "workflows", "release.yml"), "build-native");
  assert.match(job, /cross-compile: \$\{\{ matrix\.settings\.cross_compile \}\}/);
});

test("the release cannot tag before the GNU artifacts ran on old glibc", () => {
  const release = readRepoFile(".github", "workflows", "release.yml");
  const smoke = workflowJobBody(release, "smoke-glibc-floor");
  assert.match(smoke, /needs: \[version, build-packages, build-native, build-cli\]/);
  assert.match(smoke, /uses: \.\/\.github\/actions\/smoke-glibc-floor-containers/);
  assert.match(smoke, /cli: \.artifacts\/cli\/vize/);
  assert.match(workflowJobBody(release, "tag"), /smoke-glibc-floor/);

  const containers = readRepoFile(
    ".github",
    "actions",
    "smoke-glibc-floor-containers",
    "action.yml",
  );
  for (const image of [/amazonlinux:2023@sha256:/, /almalinux:8@sha256:/, /centos:7@sha256:/]) {
    assert.match(containers, image);
  }
});
