import assert from "node:assert/strict";
import path from "node:path";
import type { Browser } from "playwright";
import { cleanup, type Evidence } from "./vrt_artifacts.ts";
import { buildLiteralHostedVrt, type LiteralTitle } from "./vrt_literal.ts";
import type { VrtSecrets } from "./vrt_secrets.ts";
import {
  startPublishedVrtSession,
  assertPublishedVrtChildrenClosed as assertChildrenClosed,
} from "./vrt_service_process.ts";
import { withTrustedVrtSurface, assertTrustedVrtSurfacesClosed } from "./vrt_trusted_surface.ts";
import {
  loopbackPermission,
  publicAddressCustody,
  authorityRefusals,
  assertRealPublicLoopback,
} from "./vrt_trusted_network.ts";
import {
  assertBlobsRevoked,
  capturePublicLiteral,
  connectPublicSession,
  currentBlobURLs,
  selectLiteral,
  wholeLiteralReports,
  literalProjectRetainer,
  persistedImagesAndReports,
} from "./vrt_trusted_ui.ts";
import { sha256 } from "./vrt_fixtures.ts";
import { publicBodyCustody, observePublicLiteralPreviews } from "./vrt_trusted_gallery.ts";
import { prepareTrustedColors } from "./vrt_trusted_color_build.ts";
import { observePublicColors } from "./vrt_trusted_colors.ts";
import { foreignSessionReports } from "./vrt_trusted_report_isolation.ts";
import {
  expectedBrowserFailures,
  assertExpectedBrowserFailures,
  consoleFailure,
  type ConsoleFailure,
} from "./vrt_trusted_failures.ts";
const titles = ["Controls", "Keys"] as const;
/** Additive source preparation. Factories/live Browser are deliberately unexecuted until reviewed. */
export async function observeTrustedInstalledVrt(
  browser: Browser,
  input: {
    consumer: string;
    binary: string;
    assetDirectory: string;
  },
  e: Evidence,
  secrets: VrtSecrets,
) {
  const started = Date.now();
  const project = path.join(input.consumer, "trusted-vrt-workspace");
  // This full partial-project retention is registered BEFORE the native builder can create files.
  const retain = literalProjectRetainer(project, e);
  let failed = false,
    primary: unknown;
  const records: unknown[] = [];
  try {
    const build = await buildLiteralHostedVrt(input.consumer, e);
    const colors = await prepareTrustedColors(build, input.consumer, e);
    await withTrustedVrtSurface(
      browser,
      {
        binary: input.binary,
        assetDirectory: input.assetDirectory,
        directory: build.directory,
        fixture: build,
        evidence: e,
        secrets,
      },
      async (surface) => {
        const { page, cdp, galleryUrl, session } = surface;
        const assertRunning = () => surface.assertRunning();
        let phase = "secure-public-previews";
        const errors: string[] = [],
          consoles: ConsoleFailure[] = [];
        const expected = expectedBrowserFailures();
        let spaces: Awaited<ReturnType<typeof publicAddressCustody>> | undefined;
        const bodies = publicBodyCustody(page, build, galleryUrl, e, () => phase);
        bodies.allowLoopback(session.endpoint);
        let second: Awaited<ReturnType<typeof startPublishedVrtSession>> | undefined;
        const releases = [
          { name: "whole-public-response-bodies", run: () => bodies.close() },
          {
            name: "second-published-session",
            run: async () => {
              if (!second) return;
              const result = await second.close();
              assert.deepEqual(result, { code: 0, signal: null, stderrBytes: 0 });
              records.push({ phase: "second-session-closed", ...result });
            },
          },
          { name: "actual-CDP-address-custody", run: async () => spaces?.close() },
        ];
        page.on("pageerror", (error) => errors.push(String(error)));
        page.on("console", (message) => {
          if (message.type() === "error") consoles.push(consoleFailure(message, phase));
        });
        let observedFailure: { error: unknown } | undefined;
        try {
          spaces = await publicAddressCustody(cdp, () => phase);
          records.push({
            phase: "caller-event-custody",
            spaces: spaces.records,
            errors,
            consoles,
            publicBodies: bodies.records,
            expectedFailures: expected.records,
          });
          assertRunning();
          const origin = new URL(galleryUrl).origin;
          const globals = await observePublicLiteralPreviews({
            page,
            build,
            gallery: galleryUrl,
            bodies,
            records,
            assertRunning,
          });
          // The preview checks return to the gallery home route. Enter the
          // actual component's VRT tab before exercising its connection form.
          await selectLiteral(page, "Controls", build, galleryUrl);
          await page.getByRole("button", { name: "VRT", exact: true }).click();
          await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).waitFor();
          phase = "denied-loopback";
          expected.deniedSession(session.endpoint);
          assert.equal(await loopbackPermission(page, cdp, "denied"), "denied");
          const beforeDeny = await persistedImagesAndReports(project);
          await page
            .getByRole("textbox", { name: "VRT endpoint", exact: true })
            .fill(session.endpoint);
          await page.getByLabel("Session token", { exact: true }).fill(session.token);
          await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
          await page
            .getByRole("alert")
            .filter({ hasText: "Loopback access is blocked." })
            .waitFor();
          assert.equal(await page.getByRole("button", { name: "Run VRT", exact: true }).count(), 0);
          const denied = await page.evaluate(
            async ({ endpoint, token }) => {
              try {
                const response = await fetch(`${endpoint}/session`, {
                  headers: { Authorization: `Bearer ${token}` },
                  mode: "cors",
                  credentials: "omit",
                  redirect: "error",
                  cache: "no-store",
                  targetAddressSpace: "loopback",
                } as RequestInit & { targetAddressSpace: "loopback" });
                await response.arrayBuffer();
                return { rejected: false, status: response.status };
              } catch (error) {
                return { rejected: error instanceof TypeError };
              }
            },
            { endpoint: session.endpoint, token: session.token },
          );
          assert.deepEqual(denied, { rejected: true });
          assert.deepEqual(await persistedImagesAndReports(project), beforeDeny);
          records.push({
            phase,
            permission: "denied",
            actualBrowserRequest: denied,
            successfulCaptureOrPNGMutation: false,
          });
          assertRunning();
          phase = "granted-loopback";
          assert.equal(await loopbackPermission(page, cdp, "granted"), "granted");
          await connectPublicSession(page, session);
          await observePublicColors({
            page,
            session,
            build: colors,
            gallery: galleryUrl,
            bodies,
            e,
            records,
            setPhase: (next) => {
              phase = next;
            },
            assertRunning,
          });
          const first: Partial<Record<LiteralTitle, Buffer[]>> = {};
          let last: Awaited<ReturnType<typeof capturePublicLiteral>> | undefined;
          for (const capturePhase of ["new", "match"] as const) {
            for (const title of titles) {
              phase = `A-${title}-${capturePhase}`;
              const other = title === "Controls" ? "Keys" : "Controls";
              const prior = await wholeLiteralReports(project, other);
              const rendered = await selectLiteral(page, title, build, galleryUrl);
              await page.getByRole("button", { name: "VRT", exact: true }).click();
              await connectPublicSession(page, session);
              last = await capturePublicLiteral({
                page,
                session,
                build,
                gallery: galleryUrl,
                title,
                phase: capturePhase,
                label: `A-${capturePhase}`,
                e,
                first: first[title],
              });
              if (capturePhase === "new") first[title] = last.baselines;
              assert.deepEqual(await wholeLiteralReports(project, other), prior);
              records.push({ ...last.receipt, phase, rendered, otherWholeReportsUnchanged: other });
              assertRunning();
              assert.deepEqual(errors, []);
            }
          }
          assert.ok(last && first.Controls && first.Keys);
          assert.equal(new Set([...first.Controls, ...first.Keys].map(sha256)).size, 5);
          records.push(
            ...(await authorityRefusals({
              page,
              cdp,
              session,
              root: project,
              artPath: path.join(project, "src/Keys.art.vue"),
              e,
              expected,
              setPhase: (next) => {
                phase = next;
              },
            })),
          );
          phase = "B-reconnect";
          second = await startPublishedVrtSession(build, galleryUrl, secrets);
          assert.notEqual(second.token, session.token);
          assert.notEqual(second.endpoint, session.endpoint);
          bodies.allowLoopback(second.endpoint);
          records.push(
            await foreignSessionReports({
              session,
              other: second,
              root: project,
              origin,
              title: last.receipt.title,
              reports: last.receipt.data.reports,
              e,
            }),
          );
          await connectPublicSession(page, second);
          for (const selector of [".vrt-summary", ".vrt-results", ".vrt-error"])
            assert.equal(await page.locator(selector).count(), 0, selector);
          await page.locator(".vrt-empty").waitFor();
          for (const title of ["Keys", "Controls"] as const) {
            phase = `B-${title}-match`;
            const other = title === "Controls" ? "Keys" : "Controls";
            const prior = await wholeLiteralReports(project, other);
            if (title === "Controls") {
              await selectLiteral(page, title, build, galleryUrl);
              await page.getByRole("button", { name: "VRT", exact: true }).click();
              await connectPublicSession(page, second);
            }
            const result = await capturePublicLiteral({
              page,
              session: second,
              build,
              gallery: galleryUrl,
              title,
              phase: "match",
              label: "B-match",
              e,
              first: first[title],
            });
            assert.deepEqual(await wholeLiteralReports(project, other), prior);
            records.push({
              ...result.receipt,
              phase,
              distinctVolatileSession: true,
              otherWholeReportsUnchanged: other,
            });
            assertRunning();
            second.assertRunning();
          }
          await e.save("trusted-public/gallery.png", await page.screenshot({ fullPage: true }));
          phase = "actual-Vue-unmount";
          const finalBlobs = await currentBlobURLs(page);
          await page.getByRole("button", { name: "Props", exact: true }).click();
          assert.equal(await page.locator(".vrt-panel").count(), 0);
          expected.revokedBlobs(finalBlobs, "actual-Vue-unmount");
          await assertBlobsRevoked(page, finalBlobs);
          records.push({ phase, oldBlobCount: finalBlobs.length, allRevoked: true });
          await bodies.settle();
          assertRealPublicLoopback(spaces.records, origin, session.endpoint);
          assert.deepEqual(errors, []);
          assertExpectedBrowserFailures(spaces.records, expected, consoles);
          records.push({
            phase: "normally-trusted-public-loopback",
            gallery: galleryUrl,
            browser: browser.version(),
            globals,
            denyGrantActual: true,
          });
        } catch (error) {
          observedFailure = { error: secrets.sanitizeError(error) };
        } finally {
          try {
            await cleanup(e, "trusted-public-caller-cleanup", releases);
          } catch (error) {
            observedFailure = {
              error: new AggregateError(
                [...(observedFailure ? [observedFailure.error] : []), secrets.sanitizeError(error)],
                "Public observation and cleanup refused",
              ),
            };
          }
        }
        if (observedFailure) throw observedFailure.error;
      },
    );
  } catch (error) {
    failed = true;
    primary = secrets.sanitizeError(error);
  } finally {
    try {
      await cleanup(e, "trusted-public-project-and-barriers", [
        { name: "all-surface-resources-closed", run: async () => assertTrustedVrtSurfacesClosed() },
        { name: "all-published-child-pipes-closed", run: async () => assertChildrenClosed() },
        { name: "retain-full-partial-native-project", run: retain },
      ]);
    } catch (error) {
      primary = failed
        ? new AggregateError(
            [primary, secrets.sanitizeError(error)],
            "Trusted caller and finalizers refused",
          )
        : secrets.sanitizeError(error);
      failed = true;
    }
  }
  try {
    await e.json("trusted-public/observations.json", {
      records,
      success: !failed,
      actualTrustedElapsedMs: Date.now() - started,
    });
  } catch (error) {
    primary = new AggregateError(
      [...(failed ? [primary] : []), secrets.sanitizeError(error)],
      "Observation retention refused",
    );
    failed = true;
  }
  if (failed) throw primary;
}
