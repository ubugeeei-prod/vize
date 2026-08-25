import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { VIZE_BIN } from "../../_helpers/apps.ts";
import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../../..");
const FIXTURE_DIR = path.resolve(__dirname, "../../_fixtures/_projects/vue2-filter-build");
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const VIZE_BUILD_BIN = resolveVizeBuildBin();

function resolveVizeBuildBin(): string {
  const candidate = process.env.VIZE_TEST_BIN;
  if (candidate == null || candidate.length === 0) return VIZE_BIN;
  if (path.isAbsolute(candidate) || (!candidate.includes("/") && !candidate.includes("\\"))) {
    return candidate;
  }
  return path.resolve(REPO_ROOT, candidate);
}

function requireVizeBuildBin(): void {
  if (path.isAbsolute(VIZE_BUILD_BIN) || VIZE_BUILD_BIN.includes("/")) {
    assert.ok(
      fs.existsSync(VIZE_BUILD_BIN),
      `vize CLI is required at ${VIZE_BUILD_BIN}. Build it with \`cargo build --profile ci -p vize --features legacy\`.`,
    );
    return;
  }
  execFileSync(VIZE_BUILD_BIN, ["--version"], { cwd: REPO_ROOT });
}

describe("Vue 2 filter build snapshots (compiler)", () => {
  before(() => {
    requireVizeBuildBin();
  });

  it("snapshots generated filter output exactly", () => {
    const outDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vue2-filter-build-"));

    try {
      const stdout = execFileSync(
        VIZE_BUILD_BIN,
        [
          "build",
          "src/**/*.vue",
          "--config",
          "vize.config.json",
          "-o",
          outDir,
          "--continue-on-error",
        ],
        {
          cwd: FIXTURE_DIR,
          timeout: 120_000,
          maxBuffer: 100 * 1024 * 1024,
        },
      ).toString();
      console.log(stdout);

      const jsFiles = fs
        .readdirSync(outDir, { recursive: true })
        .map((entry) => String(entry))
        .filter((entry) => entry.endsWith(".js"))
        .sort();

      assert.deepEqual(jsFiles, ["LegacyFilters.js"]);

      const content = fs.readFileSync(path.join(outDir, "LegacyFilters.js"), "utf-8");
      assertParsesAsModule(content, "LegacyFilters.js");

      for (const expected of [
        "resolveFilter as _resolveFilter",
        'const _filter_normalizeId = _resolveFilter("normalizeId")',
        'const _filter_currency = _resolveFilter("currency")',
        'const _filter_suffix = _resolveFilter("suffix")',
        'const _filter_statusFilter = _resolveFilter("statusFilter")',
        "_toDisplayString(_filter_normalizeId($data.order.id))",
        '_filter_suffix(_filter_currency($data.amount,"$")," USD")',
        "type: _filter_statusFilter($data.order.status)",
        "value: _filter_currency($data.amount)",
      ]) {
        assert.equal(content.includes(expected), true, `${expected}\n${content}`);
      }

      for (const forbidden of [
        "$data.order.id | _ctx.normalizeId",
        '$data.amount | _ctx.currency("$") | _ctx.suffix(" USD")',
        "$data.order.status | _ctx.statusFilter",
      ]) {
        assert.equal(content.includes(forbidden), false, `${forbidden}\n${content}`);
      }

      assertSnapshot(SNAPSHOT_DIR, "vue2-filters-legacy-filters", content);
    } finally {
      fs.rmSync(outDir, { recursive: true, force: true });
    }
  });
});
