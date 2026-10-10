import assert from "node:assert/strict";
import { createHash, X509Certificate } from "node:crypto";
import { access, readFile, writeFile } from "node:fs/promises";
import { createConnection } from "node:net";
import path from "node:path";
import type { TestContext } from "node:test";
import * as tls from "node:tls";
import { parseArgs } from "./cli/index.ts";
import { startHostedVrtSession } from "./cli/serve.ts";
import {
  createSecureHost,
  trustSecureHost,
  type SecureHost,
} from "./hosted-vrt-network.fixtures.ts";

type Finalizer = () => void | Promise<void>;

/** Attempt every owned release in the supplied order; retain all cleanup failures. */
export async function attemptHostedCleanup(finalizers: readonly Finalizer[]): Promise<void> {
  const errors: unknown[] = [];
  for (const release of finalizers) {
    try {
      await release();
    } catch (error) {
      errors.push(error);
    }
  }
  if (errors.length === 1) throw errors[0];
  if (errors.length > 1) throw new AggregateError(errors, "Hosted fixture cleanup failed");
}

/** Register before acquisition; add each release immediately after taking ownership. */
export function registerHostedCleanup(t: Pick<TestContext, "after">): (release: Finalizer) => void {
  const finalizers: Finalizer[] = [];
  t.after(() => attemptHostedCleanup([...finalizers].reverse()));
  return (release) => {
    finalizers.push(release);
  };
}

async function assertListenerClosed(origin: string): Promise<void> {
  const address = new URL(origin);
  const error = await new Promise<Error>((resolve, reject) => {
    const socket = createConnection({ host: address.hostname, port: Number(address.port) });
    socket.setTimeout(5000, () => {
      socket.destroy();
      reject(new Error("Closed hosted fixture listener probe timed out"));
    });
    socket.once("connect", () => {
      socket.destroy();
      reject(new Error("Hosted fixture listener remains open"));
    });
    socket.once("error", resolve);
  });
  assert.equal(Reflect.get(error, "code"), "ECONNREFUSED");
}

/** Real refusal and real server-close failure; no capture, browser, response or trust bypass. */
export async function proveHostedCleanupFailures(
  directory: string,
  root: string,
  output: string,
): Promise<void> {
  // PEM order/text can normalize; compare every complete DER byte string and multiplicity.
  const identities = (certificates: readonly string[]) =>
    certificates.map((pem) => new X509Certificate(pem).raw.toString("base64")).sort();
  const sameIdentities = (actual: readonly string[], expected: readonly string[]) =>
    actual.length === expected.length && actual.every((der, index) => der === expected[index]);
  const originalTrust = identities(tls.getCACertificates("default"));
  const verifyAddedTrust = (certificate: Buffer) => {
    const actual = identities(tls.getCACertificates("default"));
    const expected = [
      ...originalTrust,
      new X509Certificate(certificate).raw.toString("base64"),
    ].sort();
    assert.ok(sameIdentities(actual, expected), "Only the generated CA may be added");
    assert.equal(
      sameIdentities(actual, originalTrust),
      false,
      "Changed trust cannot count as restored",
    );
  };
  const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
  const startupFinalizers: Finalizer[] = [];
  let startupHost: SecureHost | undefined;
  let startupMessage = "";
  let startupCertificate = "";
  let sessionCreated = false;
  try {
    startupHost = await createSecureHost(directory);
    const owned = startupHost;
    startupFinalizers.push(() => owned.close());
    const restoreTrust = await trustSecureHost(owned);
    startupFinalizers.push(restoreTrust);
    const certificate = await readFile(owned.certificatePath);
    startupCertificate = hash(certificate);
    verifyAddedTrust(certificate);
    const options = parseArgs([
      "serve",
      "--gallery-url",
      `${owned.origin}/built/cleanup-missing-gallery/`,
      "--output",
      path.join(root, "cleanup-startup-refusal"),
    ]);
    await assert.rejects(
      async () => {
        try {
          const session = await startHostedVrtSession(options, owned.spki);
          sessionCreated = true;
          startupFinalizers.push(() => session.close());
        } catch (error) {
          assert.ok(error instanceof Error);
          startupMessage = error.message;
          throw error;
        }
      },
      { message: "Hosted gallery manifest: HTTP 404" },
    );
  } finally {
    await attemptHostedCleanup([...startupFinalizers].reverse());
  }
  assert.ok(startupHost);
  assert.equal(sessionCreated, false);
  assert.deepEqual(startupHost.requests, [
    { method: "GET", pathname: "/built/cleanup-missing-gallery/api/static.json", status: 404 },
  ]);
  await assertListenerClosed(startupHost.origin);
  await assert.rejects(access(startupHost.certificatePath), { code: "ENOENT" });
  assert.ok(
    sameIdentities(identities(tls.getCACertificates("default")), originalTrust),
    "Original CA DER bytes and multiplicities must be restored",
  );

  const remainingFinalizers: Finalizer[] = [];
  let remainingHost: SecureHost | undefined;
  let naturalCleanupError: unknown;
  let caughtCleanupError: unknown;
  let finalized = false;
  let remainingCertificate = "";
  try {
    remainingHost = await createSecureHost(directory);
    const owned = remainingHost;
    remainingFinalizers.push(() => owned.close());
    const restoreTrust = await trustSecureHost(owned);
    remainingFinalizers.push(restoreTrust);
    const certificate = await readFile(owned.certificatePath);
    remainingCertificate = hash(certificate);
    verifyAddedTrust(certificate);
    const alreadyClosed = startupHost;
    finalized = true;
    try {
      await attemptHostedCleanup([
        async () => {
          try {
            await alreadyClosed.close();
          } catch (error) {
            naturalCleanupError = error;
            throw error;
          }
        },
        ...remainingFinalizers.slice().reverse(),
      ]);
    } catch (error) {
      caughtCleanupError = error;
    }
  } finally {
    if (!finalized) await attemptHostedCleanup([...remainingFinalizers].reverse());
  }
  assert.ok(naturalCleanupError instanceof Error);
  assert.equal(Reflect.get(naturalCleanupError, "code"), "ERR_SERVER_NOT_RUNNING");
  assert.equal(caughtCleanupError, naturalCleanupError);
  assert.ok(remainingHost);
  await assertListenerClosed(remainingHost.origin);
  await assert.rejects(access(remainingHost.certificatePath), { code: "ENOENT" });
  assert.ok(
    sameIdentities(identities(tls.getCACertificates("default")), originalTrust),
    "Original CA DER bytes and multiplicities must be restored",
  );
  await writeFile(
    path.join(output, "hosted-vrt-cleanup.json"),
    JSON.stringify(
      {
        scope:
          "Real fixture acquisition/refusal and cleanup failure; no public gallery/capture acceptance",
        startup: {
          message: startupMessage,
          requests: startupHost.requests,
          sessionCreated,
          listenerClosed: true,
          certificateRemoved: true,
          certificateSHA256: startupCertificate,
        },
        cleanup: {
          naturalErrorCode: Reflect.get(naturalCleanupError, "code"),
          originalSingleErrorPreserved: caughtCleanupError === naturalCleanupError,
          remainingListenerClosed: true,
          remainingCertificateRemoved: true,
          certificateSHA256: remainingCertificate,
        },
        currentThreadCARestored: true,
        originalCACertificateCount: originalTrust.length,
        completeDERBytesAndMultiplicityEqual: true,
        actualChangedTrustRejectedAsRestored: true,
        originalCAWholeDERListSHA256: hash(JSON.stringify(originalTrust)),
        restoredCAWholeDERListSHA256: hash(
          JSON.stringify(identities(tls.getCACertificates("default"))),
        ),
        PEMOrderAndTextAreNotEqualityCriteria: true,
      },
      null,
      2,
    ) + "\n",
  );
}
