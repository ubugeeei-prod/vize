import fs from "node:fs";
import { promises as files } from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import {
  ciRequestPath,
  readCiRequest,
} from "../../npm/native/scripts/formatter-history-ci-request.mjs";
import { ciSourceIdentity } from "../../npm/native/scripts/formatter-history-ci-build.mjs";
import { collectFailureCssom } from "./vuefes-failure-cssom.mjs";
import { recordFailureTransport } from "./vuefes-failure-transport.mjs";
import { writeFailureJson } from "./vuefes-failure-json.mjs";

const nativeDir = fileURLToPath(new URL("../../npm/native/", import.meta.url));
export function vuefesFailureIntent() {
  try {
    if (!fs.existsSync(ciRequestPath(nativeDir))) return null;
    return readCiRequest(nativeDir, ciSourceIdentity(nativeDir));
  } catch (error) {
    return { refused: String(error) };
  }
}
export function failureSidecarBudget(startedAt, declaredTimeout, now = performance.now()) {
  const callbackRemaining = declaredTimeout - (now - startedAt);
  return {
    callbackRemaining,
    milliseconds: callbackRemaining > 35_000 ? 5_000 : 0,
    reserve: 30_000,
    authority:
      "callback-start approximation; excludes pre-callback fixture setup, not framework deadline",
  };
}

export function createVuefesFailureSidecar(pages, metadata, intent = vuefesFailureIntent()) {
  if (!intent) return null;
  try {
    return prepareSidecar(pages, metadata, intent);
  } catch (error) {
    // Refused observer setup cannot replace the original navigation or matcher.
    console.error(`VueFes failure sidecar setup refused: ${String(error)}`);
    return null;
  }
}
function prepareSidecar(pages, metadata, intent) {
  if (intent.refused) {
    return {
      stop() {},
      async failed() {
        console.error(`VueFes failure sidecar refused: ${intent.refused}`);
      },
    };
  }
  const recorders = pages.map(({ page }) => recordFailureTransport(page));
  return {
    stop() {
      for (const recorder of recorders) recorder.stop();
    },
    async failed(originalError, options = {}) {
      const failedAt = performance.now();
      const pendingAtFailure = recorders.map((recorder) => recorder.stop());
      const budget = failureSidecarBudget(metadata.callbackStartedAt, metadata.declaredTimeout);
      const controller = new AbortController();
      const signal = controller.signal;
      const expires = failedAt + budget.milliseconds;
      const directory = path.join(metadata.outputDir, `retry-${metadata.retry}`);
      const status = {
        schema: "vize.vuefes.post-matcher-failure-sidecar",
        version: 1,
        complete: false,
        missing: budget.milliseconds
          ? "capture has not finalized"
          : "diagnostic budget unavailable",
        boundary:
          originalError?.matcherResult && String(originalError).includes("visual diff ratio")
            ? "original PNG matcher failed; later capture is not atomic screenshot-time authority"
            : "expectVisualParity threw; screenshot/PNG transport boundary has no matcher authority",
        originalError: String(originalError?.stack ?? originalError),
        intent,
        budget,
        callbackStartedAt: metadata.callbackStartedAt,
        failedAt,
        pendingAtFailure,
        transportAtFailure: recorders.map((recorder, index) => ({
          name: pages[index].name,
          requests: recorder.inventory(),
        })),
      };
      const freeze = () => recorders.map((recorder) => recorder.freeze());
      if (!budget.milliseconds) {
        freeze();
        console.error("VueFes failure sidecar refused: diagnostic budget unavailable");
        return;
      }
      const assertLive = () => {
        signal.throwIfAborted();
        if (performance.now() >= expires) throw new Error("diagnostic budget expired");
      };
      const persist = options.persist ?? writeFailureJson;
      const publish = async (name, value) => {
        assertLive();
        const file = path.join(directory, name);
        await persist(`${file}.partial`, value, signal);
        assertLive();
        // Only a live attempt can finalize; late writes keep their .partial suffix.
        fs.renameSync(`${file}.partial`, file);
      };
      const operation = (async () => {
        await files.mkdir(directory, { recursive: true });
        await publish("status.json", status);
        const dom = await Promise.all(
          pages.map(async ({ page, name }) => {
            const startedAt = performance.now();
            try {
              const packet = await page.evaluate(collectFailureCssom);
              return { name, startedAt, finishedAt: performance.now(), packet };
            } catch (error) {
              return {
                name,
                startedAt,
                finishedAt: performance.now(),
                complete: false,
                missing: String(error),
              };
            }
          }),
        );
        assertLive();
        for (let index = 0; index < pages.length; index++) {
          await publish(`${pages[index].name}-cssom.json`, dom[index]);
        }
        // Leave time to persist explicit pending-body refusals within the same5s budget.
        let bodyTimer;
        await Promise.race([
          Promise.all(recorders.map((recorder) => recorder.finish())),
          new Promise((resolve) => {
            bodyTimer = setTimeout(resolve, Math.max(0, expires - performance.now() - 2_000));
          }),
        ]).finally(() => clearTimeout(bodyTimer));
        assertLive();
        const transport = freeze();
        for (let index = 0; index < pages.length; index++) {
          const { name } = pages[index];
          for (const entry of transport[index].requests) {
            if (!entry.body) continue;
            assertLive();
            const file = `${name}-response-${entry.id}.bin`;
            const body = entry.body;
            entry.body = {
              file,
              bytes: body.length,
              sha256: createHash("sha256").update(body).digest("hex"),
            };
            await files.writeFile(path.join(directory, `${file}.partial`), body, {
              flag: "wx",
              signal,
            });
            assertLive();
            fs.renameSync(path.join(directory, `${file}.partial`), path.join(directory, file));
          }
          await publish(`${name}-transport.json`, transport[index]);
        }
        const missing = transport.flatMap((packet, index) => [
          ...packet.failures.map((reason) => ({ page: pages[index].name, reason })),
          ...packet.requests
            .filter((entry) => !entry.complete)
            .map((entry) => ({
              page: pages[index].name,
              request: entry.id,
              reason: entry.missing,
            })),
        ]);
        for (const entry of dom) {
          if (!entry.packet?.complete)
            missing.push({ page: entry.name, reason: entry.missing ?? entry.packet?.omissions });
        }
        await publish("final.json", {
          ...status,
          complete:
            transport.every((packet) => packet.complete) &&
            dom.every((entry) => entry.packet?.complete),
          missing,
          finishedAt: performance.now(),
        });
      })();
      // Quarantine outstanding evaluation/body/write promises; context.close cancels them.
      void operation.catch(() => {});
      let timer;
      try {
        await Promise.race([
          operation,
          new Promise((_, reject) => {
            timer = setTimeout(
              () => reject(new Error("diagnostic budget expired")),
              budget.milliseconds,
            );
          }),
        ]);
      } catch (error) {
        console.error(`VueFes failure sidecar incomplete: ${String(error)}`);
      } finally {
        clearTimeout(timer);
        controller.abort();
        freeze();
      }
    },
  };
}
