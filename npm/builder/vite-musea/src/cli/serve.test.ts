import assert from "node:assert/strict";
import test from "node:test";
import { parseArgs } from "./index.ts";
import { hostedCertificateArguments } from "../vrt/hosted-navigation.ts";

void test("the published CLI explicitly selects a fixed-gallery VRT session", () => {
  const options = parseArgs(["serve", "--gallery-url", "https://gallery.example/built/gallery/"]);
  assert.equal(options.command, "serve");
  assert.equal(options.galleryUrl, "https://gallery.example/built/gallery/");
  assert.equal(Object.hasOwn(options, "certificateSpki"), false);
});

void test("ordinary runner launches add no arguments and isolated trust accepts one exact SPKI", () => {
  assert.deepEqual(hostedCertificateArguments(undefined), []);
  const pin = Buffer.alloc(32, 7).toString("base64");
  assert.deepEqual(hostedCertificateArguments(pin), [
    `--ignore-certificate-errors-spki-list=${pin}`,
  ]);
  for (const invalid of ["", "--disable-web-security", "a".repeat(43), "a".repeat(43) + "=,other"])
    assert.throws(() => hostedCertificateArguments(invalid), /Invalid hosted certificate SPKI/);
});
