import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

interface StockIdentity {
  version: string;
  entrySha256: string;
}
interface Fixture {
  id: string;
  input: string;
  expected: string;
  inputSha256: string;
  expectedSha256: string;
  options: Record<string, unknown>;
  stock: {
    original: unknown;
    expected: unknown;
    runtime: { id: string; state: unknown; original: unknown; expected: unknown }[];
  };
}
interface StockObserver {
  identities: Record<string, StockIdentity>;
  compilePacket(source: string, id: string): unknown;
  observePacket(packet: unknown, state: unknown, id: string): Promise<unknown>;
}
interface BuildProvider {
  expectedBuildIdentity(root: string): { binaryPath: string; sourceRevision: string };
  validateBuildReceipt(receipt: unknown, expected: unknown): void;
}
interface StockProvider {
  createChildWidthStockObserver(root: string): Promise<StockObserver>;
}
interface Observation {
  source: string;
  sourceSha256: string;
  packet: unknown;
  runtime: { id: string; state: unknown; result: unknown; error: unknown }[];
  qualified: boolean;
}
interface Command {
  args: string[];
  status: number | null;
  signal: string | null;
  error?: string;
  stdout: ReturnType<typeof snapshot>;
  stderr: ReturnType<typeof snapshot>;
  file?: ReturnType<typeof snapshot>;
  config?: ReturnType<typeof snapshot>;
  after?: Observation;
}
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const authority =
  "tests/_fixtures/differential/formatter-regressions/closing-child-width-7876/references.json";
const sha = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const snapshot = (bytes: Buffer) => ({
  sha256: sha(bytes),
  bytes: bytes.length,
  base64: bytes.toString("base64"),
});

// Provider spelling may migrate independently of the immutable author's raw
// MJS custody. Record the actual provider source used by this source-built run.
function provider(stem: string) {
  const typed = `${stem}.ts`;
  const relative = fs.existsSync(path.join(root, typed)) ? typed : `${stem}.mjs`;
  const source = fs.readFileSync(path.join(root, relative));
  return {
    path: relative,
    sha256: sha(source),
    url: pathToFileURL(path.join(root, relative)).href,
  };
}

void test("actual closing CLI preserves thirteen whole stock DOM/text/SSR/event vectors", async (t) => {
  const raw = fs.readFileSync(path.join(root, authority));
  assert.equal(sha(raw), "f57812d0f91255208eca609b3ac7c985b2fbb7c6cfa7f284c4202544b9a23b74");
  const corpus = JSON.parse(raw.toString()) as {
    cases: Fixture[];
    provider: { stock: Record<string, StockIdentity> };
  };
  assert.equal(corpus.cases.length, 13);
  const providers = {
    build: provider("tests/differential/build-receipt"),
    stock: provider("tests/tooling/support/formatter-child-width-stock"),
  };
  const build = (await import(providers.build.url)) as BuildProvider;
  const stockProvider = (await import(providers.stock.url)) as StockProvider;
  const identity = build.expectedBuildIdentity(root);
  const receipt: unknown = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
  );
  build.validateBuildReceipt(receipt, identity);
  const stock = await stockProvider.createChildWidthStockObserver(root);
  for (const [id, expected] of Object.entries(corpus.provider.stock)) {
    assert.equal(stock.identities[id].version, expected.version, `${id}: stock version`);
    assert.equal(
      stock.identities[id].entrySha256,
      expected.entrySha256,
      `${id}: stock whole entry`,
    );
  }
  const binary = path.join(root, identity.binaryPath);
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-closing-width-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const report = {
    identity,
    receipt,
    providers,
    stockIdentities: stock.identities,
    referenceSha256: sha(raw),
    commandCount: 0,
    packetCount: 0,
    runtimeStateCount: 0,
    observations: [] as {
      id: string;
      before?: Observation;
      reference?: Observation;
      cli: Command[];
    }[],
  };
  const reportPath = path.join(root, "target/differential/formatter-closing-width.json");
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  async function qualify(source: string, fixture: Fixture, phase: "original" | "expected") {
    const observed: Observation = {
      source,
      sourceSha256: sha(source),
      packet: stock.compilePacket(source, fixture.id),
      runtime: [],
      qualified: false,
    };
    report.packetCount++;
    return {
      observed,
      async judge() {
        persist();
        assert.equal(source, phase === "original" ? fixture.input : fixture.expected);
        // These sealed packets include the descriptor's whole first LF/CRLF;
        // no author trimming recipe or generated-code normalization is used.
        assert.deepEqual(observed.packet, fixture.stock[phase], `${fixture.id}: whole ${phase}`);
        for (const control of fixture.stock.runtime) {
          const state = {
            id: control.id,
            state: control.state,
            result: null as unknown,
            error: null as unknown,
          };
          observed.runtime.push(state);
          persist();
          try {
            state.result = await stock.observePacket(observed.packet, control.state, fixture.id);
            report.runtimeStateCount++;
            persist();
            assert.deepEqual(
              state.result,
              control[phase],
              `${fixture.id}: whole ${phase}/${control.id}`,
            );
          } catch (error) {
            state.error =
              error instanceof Error
                ? { message: error.message, stack: error.stack }
                : String(error);
            persist();
            throw error;
          }
        }
        assert.equal(observed.runtime.length, 4);
        observed.qualified = true;
        persist();
      },
    };
  }
  for (const fixture of corpus.cases) {
    assert.equal(sha(fixture.input), fixture.inputSha256);
    assert.equal(sha(fixture.expected), fixture.expectedSha256);
    const row: (typeof report.observations)[number] = { id: fixture.id, cli: [] };
    report.observations.push(row);
    const before = await qualify(fixture.input, fixture, "original");
    row.before = before.observed;
    await before.judge();
    const reference = await qualify(fixture.expected, fixture, "expected");
    row.reference = reference.observed;
    await reference.judge();
    const project = path.join(scratch, fixture.id);
    fs.mkdirSync(project);
    const filename = "App.vue";
    const file = path.join(project, filename);
    const configPath = path.join(project, "vize.config.json");
    const config = Buffer.from(`${JSON.stringify({ formatter: fixture.options })}\n`);
    fs.writeFileSync(configPath, config);
    fs.writeFileSync(file, fixture.input);
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const changed = pass < 2 && fixture.input !== fixture.expected;
      const expected = pass === 0 ? fixture.input : fixture.expected;
      const args = ["fmt", mode, filename];
      const output = spawnSync(binary, args, { cwd: project, timeout: 60000 });
      const command: Command = {
        args,
        status: output.status,
        signal: output.signal,
        error: output.error?.message,
        stdout: snapshot(output.stdout ?? Buffer.alloc(0)),
        stderr: snapshot(output.stderr ?? Buffer.alloc(0)),
      };
      row.cli.push(command);
      report.commandCount++;
      persist();
      const actual = fs.readFileSync(file);
      const actualConfig = fs.readFileSync(configPath);
      command.file = snapshot(actual);
      command.config = snapshot(actualConfig);
      persist();
      assert.ifError(output.error);
      assert.equal(output.signal, null);
      assert.equal(output.status, Number(changed && mode === "--check"), fixture.id);
      assert.deepEqual(output.stdout, Buffer.alloc(0));
      const detail = changed
        ? `${mode === "--check" ? "Would reformat" : "Reformatted"}: ${filename}\n`
        : "";
      const summary =
        mode === "--check"
          ? `Checked 1 file(s)\n  1 file(s) ${changed ? "would be reformatted" : "already formatted"}\n`
          : `Formatted 1 file(s)\n  1 file(s) ${changed ? "reformatted" : "unchanged"}\n`;
      assert.deepEqual(output.stderr, Buffer.from(`Found 1 file(s)\n${detail}\n${summary}`));
      assert.deepEqual(actual, Buffer.from(expected));
      assert.deepEqual(actualConfig, config);
      const after = await qualify(actual.toString(), fixture, pass === 0 ? "original" : "expected");
      command.after = after.observed;
      await after.judge();
    }
  }
  assert.equal(report.commandCount, 65);
  assert.equal(report.packetCount, 91);
  assert.equal(report.runtimeStateCount, 364);
});
