#!/usr/bin/env node
// A production stdio Maestro session. The isolated resident example remains a
// separate measurement and does not establish this process-tree acceptance.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { LspSession } from "../../../../tests/tooling/support/lsp/session.ts";
import { componentSource, documentUri, prepareWorkspace } from "./lib/lsp-resource-workspace.mjs";
import { createSampler, mib } from "./lib/lsp-resource-sampler.mjs";
import { bodyEditChurn, controlledEdits, metadataCompletion } from "./lib/lsp-resource-witness.mjs";

const args = process.argv.slice(2);
const flag = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
const out = path.resolve(flag("--out", "lsp-resource.json"));
const files = Number(flag("--files", "10000"));
const openFiles = Number(flag("--open-files", String(files)));
const warmProviders = Number(flag("--warm-providers", String(openFiles)));
const runs = Number(flag("--runs", "1"));
const idleSeconds = Number(flag("--idle-seconds", "10"));
const preset = flag("--preset", "linux-x64-ci");
await measure();

async function measure() {
  assert.ok(Number.isInteger(files) && files >= 2 && files % 2 === 0);
  assert.ok(
    Number.isInteger(openFiles) && openFiles >= 2 && openFiles <= files && openFiles % 2 === 0,
  );
  assert.ok(Number.isInteger(runs) && runs > 0 && idleSeconds >= 10);
  assert.ok(
    Number.isInteger(warmProviders) &&
      warmProviders >= 2 &&
      warmProviders <= openFiles &&
      warmProviders % 2 === 0,
  );
  const server = fs.realpathSync(flag("--server", "target/ci-opt/vize"));
  process.env.VIZE_LSP_BIN = server;
  const measurement = {
    scope: "production Maestro LSP with all descendants, including Corsa; harness excluded",
    preset,
    workspace_files: files,
    open_files: openFiles,
    unique_imported_providers: warmProviders,
    provider_coverage: `${warmProviders} of ${openFiles} open providers receive prop completions`,
    runs,
    completed_runs: 0,
    complete: false,
    services: { editor: true, ecosystem: true, lint: true, typecheck: true },
    server,
    server_binary_mib: mib(fs.statSync(server).size),
    build: "ci-opt: release, thin LTO, 16 codegen units",
    sampler: "Linux /proc process-tree summed RSS every 50 ms, from spawn through idle",
    idle_seconds: idleSeconds,
    acceptance:
      "record-only: broader workload than the separately enforced nine-file TS-44 baseline",
    command: [process.execPath, ...process.argv.slice(1)],
    github_run:
      process.env.GITHUB_RUN_ID &&
      `https://github.com/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    revision: process.env.GITHUB_SHA,
    per_run: [],
  };
  const save = () => fs.writeFileSync(out, `${JSON.stringify(measurement, null, 2)}\n`);
  fs.mkdirSync(path.dirname(out), { recursive: true });
  save();
  for (let run = 0; run < runs; run++) {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-real-lsp-resource-"));
    const started = performance.now();
    const dependencies = prepareWorkspace(workspace, files);
    const session = new LspSession();
    const sampler = createSampler(session.processId);
    const result = {
      run: run + 1,
      dependencies,
      opened_files: 0,
      metadata_requests: 0,
      successful_unique_provider_requests: 0,
      resident_document_witnesses: 0,
      diagnostic_publications: 0,
    };
    session.notificationObservers.push((method) => {
      if (method === "textDocument/publishDiagnostics") result.diagnostic_publications++;
    });
    measurement.active_run = result;
    save();
    try {
      await session.initialize(workspace, measurement.services);
      result.initialized_ms = performance.now() - started;
      const open = (index) => {
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri: documentUri(workspace, index),
            languageId: "vue",
            version: 1,
            text: componentSource(index),
          },
        });
        result.opened_files++;
      };
      open(0);
      open(1);
      await metadataCompletion(session, documentUri(workspace, 1), componentSource(1));
      result.controlled_edits_scope = "two open files before the bulk residency workload";
      result.body_edit_churn = await bodyEditChurn(session, workspace);
      result.controlled_edits = await controlledEdits(
        session,
        workspace,
        result.body_edit_churn.final_version,
      );
      save();
      // Bulk notifications avoid repeated full native analysis between batches.
      // Every buffer stays open; structural replies below prove it is resident.
      for (let batch = 0; batch < openFiles; batch += 32) {
        const end = Math.min(batch + 32, openFiles);
        for (let index = Math.max(batch, 2); index < end; index++) open(index);
        await new Promise((resolve) => setImmediate(resolve));
      }
      for (let batch = 0; batch < openFiles; batch += 32) {
        const end = Math.min(batch + 32, openFiles);
        await Promise.all(
          Array.from({ length: end - batch }, async (_, offset) => {
            const symbols = await session.request(
              "textDocument/documentSymbol",
              { textDocument: { uri: documentUri(workspace, batch + offset) } },
              300_000,
            );
            assert.ok(symbols?.some((symbol) => symbol.name === "script setup"));
            assert.ok(symbols.some((symbol) => symbol.name === "template"));
            result.resident_document_witnesses++;
          }),
        );
        if (end % 256 === 0 || end === openFiles) {
          result.warming_elapsed_ms = performance.now() - started;
          const current = sampler.sample();
          result.current_rss_mib = mib(current.rss_bytes);
          result.current_processes = current.processes;
          console.error(
            `run ${run + 1}: ${end}/${openFiles} resident document witnesses; ${result.current_rss_mib} MiB`,
          );
          measurement.active_run = result;
          save();
        }
      }
      const pairs = warmProviders / 2;
      for (let pair = 0; pair < pairs; pair++) {
        const first = pairs === 1 ? 0 : Math.floor((pair * (openFiles / 2 - 1)) / (pairs - 1)) * 2;
        for (const index of [first, first + 1]) {
          result.metadata_requests++;
          await metadataCompletion(session, documentUri(workspace, index), componentSource(index));
          result.successful_unique_provider_requests++;
        }
        measurement.active_run = result;
        save();
      }
      const idleStart = performance.now();
      const before = sampler.sample();
      await new Promise((resolve) => setTimeout(resolve, idleSeconds * 1000));
      const after = sampler.sample();
      assert.ok(
        after.processes.some((entry) => entry.pid !== session.processId),
        "typecheck acceptance requires the live Corsa descendant to be measured",
      );
      result.rss_idle_mib = mib(after.rss_bytes);
      result.idle_cpu_pct =
        ((after.cpu_seconds - before.cpu_seconds) / ((performance.now() - idleStart) / 1000)) * 100;
      result.idle_processes = after.processes;
      result.elapsed_ms = performance.now() - started;
    } catch (error) {
      result.error = String(error);
      result.server_stderr = session.stderrText;
      throw error;
    } finally {
      const { peak, samples } = sampler.stop();
      result.rss_peak_mib = mib(peak.rss_bytes);
      result.peak_processes = peak.processes;
      result.resource_samples = samples;
      measurement.per_run.push(result);
      delete measurement.active_run;
      save();
      await session.shutdown().catch(async () => session.kill());
      fs.rmSync(workspace, { force: true, recursive: true });
    }
    measurement.completed_runs++;
    save();
  }
  measurement.metrics = Object.fromEntries(
    ["rss_peak_mib", "rss_idle_mib", "idle_cpu_pct"].map((key) => [
      key,
      Math.max(...measurement.per_run.map((run) => run[key])),
    ]),
  );
  measurement.complete = true;
  save();
  console.log(JSON.stringify({ out, metrics: measurement.metrics, files, openFiles, runs }));
}
