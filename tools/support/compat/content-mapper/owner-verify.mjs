import { assert, readFile, json, path, sha256, parseFrames } from "./owner-process.mjs";

const load = async (file) => JSON.parse(await readFile(file, "utf8"));
const bytes = (text, start, length) =>
  Buffer.from(text)
    .subarray(start, start + length)
    .toString();
const sorted = (rows) => rows.map((row) => JSON.stringify(row)).sort();
function mappingPayloads(transform) {
  return sorted(
    transform.mappings.map(([generated, length, original, originalLength, kind, features]) => [
      original,
      originalLength,
      kind,
      features,
      bytes(transform.text, generated, length),
    ]),
  );
}
function linkPayloads(transform) {
  return sorted(
    (transform.semanticLinks ?? []).map((link) => {
      const { sourceStart, sourceLength, targetStart, targetLength, kind } = link;
      const target = Buffer.from(transform.text);
      const before = target.subarray(0, targetStart);
      const end = target.indexOf(10, targetStart + targetLength);
      const line = target
        .subarray(before.lastIndexOf(10) + 1, end < 0 ? target.length : end)
        .toString();
      const owners = transform.mappings
        .filter(([start, length]) => start === sourceStart && length === sourceLength)
        .map(([, , start, length, rowKind, features]) => [start, length, rowKind, features]);
      return [
        kind,
        sorted(owners),
        bytes(transform.text, sourceStart, sourceLength),
        bytes(transform.text, targetStart, targetLength),
        line,
      ];
    }),
  );
}
export async function verify(root) {
  await load(path.join(root, "capture-complete.json"));
  const old = path.join(root, "old");
  const candidate = path.join(root, "new");
  const identity = await load(path.join(root, "identity.json"));
  assert.match(identity.baseline, /^[0-9a-f]{40}$/);
  assert.match(identity.candidate, /^[0-9a-f]{40}$/);
  const provider = await load(path.join(root, "provider.json"));
  assert.equal(provider.source, "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799");
  assert.equal(provider.binary, sha256(await readFile(path.join(root, "tsgo"))));
  assert(provider.libraries.length > 0);
  for (const phase of [old, candidate]) {
    const binaries = await load(path.join(phase, "binaries.json"));
    for (const [name, binary] of Object.entries(binaries)) {
      const filename = name === "cli" ? "vize" : "owner-harness";
      assert.equal(binary.sha256, sha256(await readFile(path.join(phase, filename))));
    }
  }
  for (let index = 0; index < 16; index++) {
    const name = `packets/cli/case-${index}`;
    const baseline = path.join(old, name);
    const current = path.join(candidate, name);
    assert.deepEqual(
      await readFile(path.join(baseline, "Owner.vue")),
      await readFile(path.join(current, "Owner.vue")),
    );
    const oldOutcome = await load(path.join(baseline, "outcome.json"));
    assert.equal(
      oldOutcome.status,
      0,
      `genuine OLD RED case-${index}: child must be wrongly suppressed`,
    );
    assert.equal((await readFile(path.join(baseline, "stdout.bin"))).length, 0);
    assert.equal((await readFile(path.join(baseline, "stderr.bin"))).length, 0);
    const outcome = await load(path.join(current, "outcome.json"));
    assert(Number.isInteger(outcome.status) && outcome.status !== 0);
    assert.equal(outcome.success, false);
    assert.equal((await readFile(path.join(current, "stderr.bin"))).length, 0);
    assert.deepEqual(
      await readFile(path.join(current, "stdout.bin")),
      await readFile(path.join(current, "expected.bin")),
    );
  }
  for (let index = 0; index < 4; index++) {
    const name = `packets/cli/calibration-${index}`;
    for (const file of ["Owner.vue", "stdout.bin", "stderr.bin", "outcome.json", "expected.bin"]) {
      assert.deepEqual(
        await readFile(path.join(old, name, file)),
        await readFile(path.join(candidate, name, file)),
        `whole calibration ${index} ${file}`,
      );
    }
    const stdout = await readFile(path.join(candidate, name, "stdout.bin"), "utf8");
    assert.deepEqual(
      await readFile(path.join(candidate, name, "stdout.bin")),
      await readFile(path.join(candidate, name, "expected.bin")),
    );
    const count = [6, 2, 5, 1][index];
    assert.equal(
      stdout.split("\n").filter((line) => line.startsWith("directives/Owner.vue(")).length,
      count,
    );
    assert.equal((stdout.match(/error TS2322:/g) ?? []).length, count - 1);
    assert.equal((stdout.match(/error TS2339:/g) ?? []).length, 1);
  }
  for (let session = 0; session < 4; session++) {
    for (let index = 0; index < 11; index++) {
      const name = `packets/lsp/session-${session}/state-${index}`;
      const oldReport = await load(path.join(old, name, "report.json"));
      const current = await load(path.join(candidate, name, "report.json"));
      assert.deepEqual(
        await readFile(path.join(old, name, "Owner.vue")),
        await readFile(path.join(candidate, name, "Owner.vue")),
      );
      const expected = await load(path.join(candidate, name, "expected.json"));
      const oldExpected = await load(path.join(candidate, name, "old-expected.json"));
      assert.deepEqual(current, expected, `NEW full LSP ${session}/${index}`);
      assert.deepEqual(oldReport, oldExpected, `OLD full LSP ${session}/${index}`);
      if ([0, 9, 10].includes(index)) {
        // These exact inherited failures are OPEN fronts, not semantic acceptance.
        assert.deepEqual(oldReport, current, `known-failing whole fallback ${session}/${index}`);
        assert.equal(current.items.filter((item) => item.code === 100036).length, 1);
      } else {
        assert(
          !current.items.some((item) => item.code === 100036),
          `no unmapped aggregate in admitted/repair case ${session}/${index}`,
        );
      }
    }
    for (const phase of [old, candidate]) {
      assert.equal(
        (await load(path.join(phase, `packets/lsp/session-${session}/shutdown.json`))).success,
        true,
      );
      assert(
        (
          await load(path.join(phase, `packets/lsp/session-${session}/server-transcript.json`))
        ).some((item) => item.direction === "server-request"),
      );
    }
  }
  const oldFrames = parseFrames(await readFile(path.join(old, "mapper/stdout.bin")));
  const newFrames = parseFrames(await readFile(path.join(candidate, "mapper/stdout.bin")));
  assert.equal(oldFrames.length, 19);
  assert.equal(newFrames.length, 19);
  for (let index = 0; index < 16; index++) {
    const before = oldFrames[index + 2].result;
    const after = newFrames[index + 2].result;
    assert.deepEqual(
      sorted(before.text.split("\n")),
      sorted(after.text.split("\n")),
      `all line multiplicities ${index}`,
    );
    assert.deepEqual(mappingPayloads(before), mappingPayloads(after), `all wire spans ${index}`);
    assert.deepEqual(linkPayloads(before), linkPayloads(after), `both semantic endpoints ${index}`);
    const {
      text: _a,
      mappings: _b,
      semanticLinks: _c,
      diagnosticDirectives: d1,
      ...rest1
    } = before;
    const { text: _d, mappings: _e, semanticLinks: _f, diagnosticDirectives: d2, ...rest2 } = after;
    assert.deepEqual(rest1, rest2);
    assert.deepEqual(d1.unusedExpectDirectiveDiagnostics, d2.unusedExpectDirectiveDiagnostics);
    assert.equal(d1.directives.length, 1);
    assert.equal(d2.directives.length, 1);
    const [token, length, start, end, policy, unused] = d2.directives[0];
    assert.deepEqual(
      [token, length, policy, unused],
      [d1.directives[0][0], d1.directives[0][1], 1, 0],
    );
    const source = await readFile(path.join(candidate, `packets/cli/case-${index}/Owner.vue`));
    const child = source.indexOf("value.bad");
    const parent = source.indexOf("<Comp");
    const parentEnd = source.indexOf(">", parent) + 1;
    const parentRows = after.mappings.filter(
      ([, , at, len]) => at < parentEnd && parent < at + len,
    );
    const childRows = after.mappings.filter(([, , at, len]) => at <= child && child < at + len);
    assert(parentRows.length > 0 && childRows.length > 0);
    assert(parentRows.every(([at, len]) => start <= at && at + len <= end));
    assert(childRows.every(([at, len]) => at + len <= start || end <= at));
    const [, , oldStart, oldEnd] = d1.directives[0];
    assert(
      before.mappings.some(
        ([at, len, original, originalLength]) =>
          original <= child &&
          child < original + originalLength &&
          oldStart <= at &&
          at + len <= oldEnd,
      ),
    );
  }
  for (const [phase, expected] of [
    [old, 101],
    [candidate, 0],
  ]) {
    for (const selector of ["cli-test", "lsp-test"]) {
      const outcome = await load(path.join(phase, selector, "outcome.json"));
      assert.equal(outcome.status, expected);
      assert.equal(outcome.signal, null);
      const stdout = await readFile(path.join(phase, selector, "stdout.bin"), "utf8");
      assert.match(stdout, /running 1 test/);
      assert(!/skipping/i.test(stdout));
      assert(!/skipping/i.test(await readFile(path.join(phase, selector, "stderr.bin"), "utf8")));
    }
  }
  const unit = await load(path.join(candidate, "unit-results.json"));
  assert.equal(unit.length, 4);
  unit.forEach((result) => {
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert(result.names.length > 0);
  });
  const stock = await load(path.join(candidate, "stock-results.json"));
  assert.equal(stock.length, 5);
  for (const result of stock) {
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert(result.names.length > 0);
    const command = await load(path.join(candidate, "stock", result.name, "run/command.json"));
    for (const alias of [
      "CORSA_PATH",
      "CORSA_EXECUTABLE",
      "TSGO_PATH",
      "TSGO_EXECUTABLE",
      "VIZE_TEST_CONTENT_MAPPER_TSGO",
    ]) {
      assert.equal(
        command.env[alias],
        identity.env.VIZE_TEST_CONTENT_MAPPER_TSGO,
        `authenticated stock provider ${result.name}/${alias}`,
      );
    }
    const stdout = await readFile(
      path.join(candidate, "stock", result.name, "run/stdout.bin"),
      "utf8",
    );
    const stderr = await readFile(
      path.join(candidate, "stock", result.name, "run/stderr.bin"),
      "utf8",
    );
    assert(!/skipping/i.test(stdout + stderr), `no stock early returns: ${result.name}`);
    assert(stdout.includes(`test result: ok. ${result.names.length} passed; 0 failed; 0 ignored;`));
    result.names.forEach((name) =>
      assert(stdout.includes(`test ${name} ... ok`), `actual stock test ${name}`),
    );
  }
  const verdict = {
    oldActualRed: 16,
    cliWhole: 16,
    calibrations: 4,
    lspWholeReportsAsserted: 44,
    boundedOwnerAcceptance: 32,
    unchangedKnownFrontiers: 12,
    overall6258SemanticAcceptance: false,
    transforms: 16,
    unitFilters: 4,
    stockRustTests: stock.map(({ name, names, pureNames, backendDependentNames }) => ({
      name,
      count: names.length,
      pure: pureNames.length,
      backendDependent: backendDependentNames.length,
    })),
  };
  await json(path.join(root, "verified.json"), verdict);
  console.log(JSON.stringify(verdict));
}
if (process.argv[1] === new URL(import.meta.url).pathname) {
  const root = path.resolve(process.argv[2]);
  try {
    await verify(root);
  } catch (error) {
    await json(path.join(root, "verification-failure.json"), {
      message: String(error),
      stack: error.stack,
    });
    throw error;
  }
}
