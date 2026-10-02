import assert from "node:assert/strict";
import {
  createHash,
  createPublicKey,
  generateKeyPairSync,
  verify,
  X509Certificate,
} from "node:crypto";
import { createRequire } from "node:module";

import { upstreamModulus, upstreamSignature } from "./node-forge-vector.ts";

type Forge = ReturnType<ReturnType<typeof createRequire>>;

export function verifyForgeSecurity(forge: Forge): void {
  const keys = generateKeyPairSync("rsa", {
    modulusLength: 2048,
    publicExponent: 3,
    privateKeyEncoding: { type: "pkcs1", format: "pem" },
    publicKeyEncoding: { type: "spki", format: "pem" },
  });
  const privateKey = forge.pki.privateKeyFromPem(keys.privateKey);
  const publicKey = forge.pki.publicKeyFromPem(keys.publicKey);
  const message = "node-forge nested DigestAlgorithm security regression";
  const digest = createHash("sha256").update(message).digest("latin1");
  const asn = (type: number, value: unknown, constructed = false) =>
    forge.asn1.create(forge.asn1.Class.UNIVERSAL, type, constructed, value);
  const nullNode = () => asn(forge.asn1.Type.NULL, "");
  function signature(parameters: unknown[], suffix = ""): string {
    const oid = asn(
      forge.asn1.Type.OID,
      forge.asn1.oidToDer(forge.oids.sha256).getBytes() + suffix,
    );
    const info = asn(
      forge.asn1.Type.SEQUENCE,
      [
        asn(forge.asn1.Type.SEQUENCE, [oid, ...parameters], true),
        asn(forge.asn1.Type.OCTETSTRING, digest),
      ],
      true,
    );
    const der = forge.asn1.toDer(info).getBytes();
    const padding = Math.ceil(privateKey.n.bitLength() / 8) - der.length - 3;
    assert.ok(padding >= 8);
    // Controlled malformed signatures use the real private RSA operation.
    // This proves default parser rejection, not a no-private-key forgery.
    return privateKey.sign(null, {
      encode: () => "\0\x01" + "\xff".repeat(padding) + "\0" + der,
    });
  }
  assert.equal(publicKey.verify(digest, signature([])), true);
  assert.equal(publicKey.verify(digest, signature([nullNode()])), true);
  const malformed = [
    signature([nullNode(), asn(forge.asn1.Type.OCTETSTRING, "garbage")]),
    signature([asn(forge.asn1.Type.OCTETSTRING, "garbage")]),
    signature([asn(forge.asn1.Type.NULL, "garbage")]),
    signature([asn(forge.asn1.Type.NULL, [asn(forge.asn1.Type.OCTETSTRING, "garbage")], true)]),
    signature([], "\x80"),
    signature([], "\x80\x80"),
  ];
  for (const value of malformed) {
    assert.throws(() => publicKey.verify(digest, value), /DigestInfo value/);
  }
  const upstreamKey = forge.pki.rsa.setPublicKey(
    new forge.jsbn.BigInteger(upstreamModulus, 16),
    new forge.jsbn.BigInteger("3"),
  );
  assert.throws(
    () =>
      upstreamKey.verify(
        createHash("sha256").update("hello world!").digest("latin1"),
        Buffer.from(upstreamSignature, "hex").toString("latin1"),
      ),
    /DigestInfo value/,
  );
  for (const algorithm of ["sha1", "sha256", "sha384", "sha512", "md5"]) {
    const md = forge.md[algorithm].create();
    md.update(message);
    const value = privateKey.sign(md);
    assert.equal(publicKey.verify(md.digest().getBytes(), value), true);
    assert.equal(
      verify(algorithm, Buffer.from(message), keys.publicKey, Buffer.from(value, "latin1")),
      true,
    );
  }
  const certificate = forge.pki.createCertificate();
  certificate.publicKey = publicKey;
  certificate.serialNumber = "01";
  certificate.validity.notBefore = new Date("2026-01-01T00:00:00Z");
  certificate.validity.notAfter = new Date("2027-01-01T00:00:00Z");
  const attributes = [{ name: "commonName", value: "localhost" }];
  certificate.setSubject(attributes);
  certificate.setIssuer(attributes);
  certificate.setExtensions([
    { name: "basicConstraints", cA: true },
    { name: "subjectAltName", altNames: [{ type: 2, value: "localhost" }] },
  ]);
  certificate.sign(privateKey, forge.md.sha256.create());
  assert.equal(certificate.verify(certificate), true);
  const pem = forge.pki.certificateToPem(certificate);
  assert.equal(new X509Certificate(pem).verify(createPublicKey(keys.publicKey)), true);
  const encrypted = forge.pki.encryptPrivateKeyInfo(
    forge.pki.wrapRsaPrivateKey(forge.pki.privateKeyToAsn1(privateKey)),
    "security-proof",
    { algorithm: "aes256" },
  );
  const decoded = forge.pki.decryptPrivateKeyInfo(encrypted, "security-proof");
  assert.equal(
    forge.pki.privateKeyToPem(forge.pki.privateKeyFromAsn1(decoded)),
    forge.pki.privateKeyToPem(privateKey),
  );
  const pfx = forge.pkcs12.toPkcs12Asn1(privateKey, [certificate], "security-proof", {
    algorithm: "3des",
  });
  const reread = forge.pkcs12.pkcs12FromAsn1(
    forge.asn1.fromDer(forge.asn1.toDer(pfx).getBytes()),
    "security-proof",
  );
  const certBag = reread.getBags({ bagType: forge.pki.oids.certBag })[forge.pki.oids.certBag][0];
  const keyBag = reread.getBags({ bagType: forge.pki.oids.pkcs8ShroudedKeyBag })[
    forge.pki.oids.pkcs8ShroudedKeyBag
  ][0];
  assert.equal(forge.pki.certificateToPem(certBag.cert), pem);
  assert.equal(forge.pki.privateKeyToPem(keyBag.key), forge.pki.privateKeyToPem(privateKey));
}
