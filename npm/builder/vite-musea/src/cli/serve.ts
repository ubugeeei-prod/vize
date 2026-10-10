import { randomBytes } from "node:crypto";
import { createServer, type ServerResponse } from "node:http";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { MuseaVrtRunner, generateVrtJsonReport, generateVrtReport } from "../vrt.js";
import {
  assertArtReportOwnership,
  attachArtReportOwner,
  resolveArtReportTarget,
} from "../vrt/report-identity.js";
import { loadHostedGallery } from "./hosted.js";
import { createVrtOptions } from "./commands.js";
import type { CliOptions } from "./index.js";
import { authorizeSession, captureInput } from "./service/auth.js";
import { SessionArtifacts } from "./service/artifacts.js";

function json(response: ServerResponse, value: unknown, status = 200): void {
  response.setHeader("Content-Type", "application/json");
  response.writeHead(status).end(JSON.stringify(value));
}

/** Local lifecycle authority; callers must keep credentials out of reports and artifacts. */
export async function startHostedVrtSession(options: CliOptions, certificateSpki?: string) {
  if (!options.galleryUrl) throw new Error("serve requires --gallery-url");
  const gallery = new URL(options.galleryUrl);
  gallery.pathname = `${gallery.pathname.replace(/\/+$/, "")}/`;
  gallery.search = "";
  gallery.hash = "";
  if (gallery.protocol !== "https:" || gallery.username || gallery.password)
    throw new Error("serve requires an HTTPS gallery without embedded credentials");
  await loadHostedGallery(gallery.href);
  const token = randomBytes(32).toString("base64url");
  const artifacts = new SessionArtifacts();
  let host = "";
  let busy: Promise<void> | undefined;
  let closing = false;
  let closeTask: Promise<void> | undefined;
  const server = createServer((request, response) => {
    if (!authorizeSession(request, response, gallery.origin, host, token)) return;
    if (closing) {
      json(response, { error: "Session is closing" }, 503);
      return;
    }
    if (request.method === "GET" && request.url === "/session") {
      json(response, { version: 1, galleryUrl: gallery.href });
      return;
    }
    if (request.method === "GET" && request.url && artifacts.send(request.url, response)) return;
    if (request.method !== "POST" || request.url !== "/capture") {
      json(response, { error: "Not found" }, 404);
      return;
    }
    if (busy) {
      json(response, { error: "Capture already running" }, 409);
      return;
    }
    busy = (async () => {
      let runner: MuseaVrtRunner | undefined;
      let outcome: { body: unknown; status: number } = {
        body: { error: "Capture incomplete" },
        status: 500,
      };
      try {
        const input = await captureInput(request);
        const hosted = await loadHostedGallery(gallery.href);
        const art = hosted.arts.find((item) => item.path === input.artPath);
        if (!art) {
          outcome = { body: { error: "Art not found" }, status: 404 };
          return;
        }
        // HTTP redirects are refused before planning; browser redirects are guarded before PNG writes.
        for (const variant of art.variants.filter((item) => !item.skipVrt)) {
          const preview = await fetch(hosted.previewUrls[art.path][variant.name], {
            redirect: "error",
            signal: AbortSignal.timeout(30000),
          });
          if (!preview.ok) throw new Error(`Hosted preview: HTTP ${preview.status}`);
          await preview.body?.cancel();
        }
        const reportDir = path.resolve(options.output, "reports");
        const identity = hosted.snapshotIdentities[art.path];
        const target = resolveArtReportTarget(
          identity,
          Object.values(hosted.snapshotIdentities),
          process.cwd(),
          reportDir,
        );
        await assertArtReportOwnership(target, process.cwd());
        runner = new MuseaVrtRunner({
          ...createVrtOptions(options),
          capture: { ...createVrtOptions(options).capture, waitForPreviewReady: true },
          previewUrls: hosted.previewUrls,
          snapshotIdentities: hosted.snapshotIdentities,
          hostedNavigation: gallery.href,
        });
        await runner.init({ hostedCertificateSpki: certificateSpki });
        const results = await runner.runAllTests([art], gallery.origin);
        const summary = runner.getSummary(results);
        if ((summary.errors ?? 0) > 0) throw new Error(results.find((item) => item.error)!.error);
        if (input.update) await runner.updateBaselines(results);
        const rawJson = attachArtReportOwner(generateVrtJsonReport(results, summary), target);
        const html = generateVrtReport(results, summary);
        await mkdir(reportDir, { recursive: true });
        await writeFile(target.jsonReportPath, rawJson);
        await writeFile(target.htmlReportPath, html);
        artifacts.clear();
        const uiResults = await artifacts.results(results);
        outcome = {
          status: 200,
          body: {
            success: true,
            results: uiResults,
            summary,
            reports: {
              json: artifacts.add(rawJson, "application/json"),
              html: artifacts.add(html, "text/html"),
            },
          },
        };
      } catch (error) {
        outcome = {
          status: 400,
          body: { error: error instanceof Error ? error.message : String(error) },
        };
      } finally {
        try {
          await runner?.close();
        } catch (error) {
          outcome = { status: 500, body: { error: `Capture cleanup failed: ${String(error)}` } };
        }
        busy = undefined;
        if (!response.destroyed) json(response, outcome.body, outcome.status);
      }
    })().catch((error) => {
      busy = undefined;
      if (!response.destroyed && !response.headersSent)
        json(response, { error: String(error) }, 500);
    });
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("Missing loopback listener");
  host = `127.0.0.1:${address.port}`;
  return {
    endpoint: `http://${host}`,
    token,
    close() {
      return (closeTask ??= (async () => {
        closing = true;
        await busy;
        artifacts.clear();
        await new Promise<void>((resolve, reject) =>
          server.close((error) => (error ? reject(error) : resolve())),
        );
      })());
    },
  };
}

export async function runServe(options: CliOptions): Promise<void> {
  const session = await startHostedVrtSession(options);
  console.log(`  VRT endpoint: ${session.endpoint}\n  Session token: ${session.token}\n`);
  const stop = () => {
    void session.close().then(
      () => {
        process.exitCode = 0;
      },
      (error) => {
        console.error("VRT session cleanup failed:", error);
        process.exitCode = 1;
      },
    );
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
}
