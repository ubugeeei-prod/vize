import assert from "node:assert/strict";
import { X509Certificate } from "node:crypto";
import fs from "node:fs";
import https from "node:https";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  assertForgePackage,
  verifyInstalledForge,
} from "../../tools/support/security/node-forge-proof.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const store = path.join(root, "node_modules/.pnpm");

test("the actual frozen installation attests every Node listhen Forge instance", () => {
  assert.ok(verifyInstalledForge(root).instances > 0);
});

for (const [name, version] of [
  ["@nuxt/cli", "3.37.0"],
  ["nitropack", "2.13.4"],
]) {
  test(
    `${name} resolves patched listhen TLS, encrypted keys and PKCS12`,
    { timeout: 30_000 },
    async () => {
      const prefix = name.replace("/", "+") + "@" + version;
      const entries = fs
        .readdirSync(store)
        .filter((entry) => entry === prefix || entry.startsWith(prefix + "_"));
      assert.ok(entries.length > 0, "actual framework consumer must be installed");
      for (const entry of entries) {
        const consumer = createRequire(
          path.join(store, entry, "node_modules", name, "package.json"),
        );
        const consumerManifest = consumer(
          path.join(store, entry, "node_modules", name, "package.json"),
        );
        assert.equal(consumerManifest.name, name);
        assert.equal(consumerManifest.version, version);
        const listenerRoot = path.dirname(path.dirname(consumer.resolve("listhen")));
        const listenerRequire = createRequire(path.join(listenerRoot, "package.json"));
        const forgeRoot = path.dirname(listenerRequire.resolve("node-forge/package.json"));
        assertForgePackage(forgeRoot);
        const forge = listenerRequire(forgeRoot);
        const { listen } = await import(
          pathToFileURL(path.join(listenerRoot, "dist/index.mjs")).href
        );
        const options = {
          hostname: "127.0.0.1",
          port: 0,
          isTest: true,
          public: false,
          showURL: false,
          autoClose: false,
        };
        const passphrase = "listhen-security-proof";
        const handle = (_request: unknown, response: import("node:http").ServerResponse) => {
          response.end(name + " patched TLS");
        };
        const generated = await listen(handle, {
          ...options,
          https: { signingKeyPassphrase: passphrase, passphrase },
        });
        let certificate: string;
        let privateKey: ReturnType<typeof forge.pki.privateKeyFromAsn1>;
        try {
          assert.match(generated.https.key, /^-----BEGIN ENCRYPTED PRIVATE KEY-----/);
          assert.equal(generated.https.passphrase, passphrase);
          await request(generated.url, name, generated.https.cert);
          certificate = generated.https.cert;
          const encrypted = forge.pki.encryptedPrivateKeyFromPem(generated.https.key);
          privateKey = forge.pki.privateKeyFromAsn1(
            forge.pki.decryptPrivateKeyInfo(encrypted, passphrase),
          );
        } finally {
          await generated.close();
        }
        const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-listhen-pfx-"));
        try {
          const pfx = forge.pkcs12.toPkcs12Asn1(
            privateKey,
            [forge.pki.certificateFromPem(certificate)],
            passphrase,
            { algorithm: "3des" },
          );
          const file = path.join(scratch, "listener.pfx");
          fs.writeFileSync(file, Buffer.from(forge.asn1.toDer(pfx).getBytes(), "latin1"));
          const restored = await listen(handle, { ...options, https: { pfx: file, passphrase } });
          try {
            assert.equal(restored.https.cert, certificate);
            assert.equal(restored.https.key, forge.pki.privateKeyToPem(privateKey));
            await request(restored.url, name, certificate);
          } finally {
            await restored.close();
          }
        } finally {
          fs.rmSync(scratch, { recursive: true, force: true });
        }
      }
    },
  );
}

function request(url: string, consumer: string, certificate: string): Promise<void> {
  return new Promise((resolve, reject) => {
    // The generated development certificate is private to this loopback law.
    const outgoing = https.get(url, { rejectUnauthorized: false, agent: false }, (response) => {
      try {
        assert.equal(response.statusCode, 200);
        const socket = response.socket as import("node:tls").TLSSocket;
        const actual = new X509Certificate(socket.getPeerCertificate().raw);
        assert.equal(actual.fingerprint256, new X509Certificate(certificate).fingerprint256);
      } catch (error) {
        reject(error);
        response.resume();
        return;
      }
      let body = "";
      response.setEncoding("utf8");
      response.on("data", (chunk) => {
        body += chunk;
      });
      response.on("error", reject);
      response.on("end", () => {
        try {
          assert.equal(body, consumer + " patched TLS");
          resolve();
        } catch (error) {
          reject(error);
        }
      });
    });
    outgoing.setTimeout(10_000, () => outgoing.destroy(new Error("loopback TLS timed out")));
    outgoing.on("error", reject);
  });
}
