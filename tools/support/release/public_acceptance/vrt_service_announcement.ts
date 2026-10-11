import assert from "node:assert/strict";
import { VrtSecrets } from "./vrt_secrets.ts";

export const publishedTokenNotice =
  "This session token is a live credential; do not share it or retain it in captured logs.";

/** Parses owned published-CLI stdout only; no raw line, credential or fingerprint is saved. */
export class PublishedVrtAnnouncement {
  #notice = false;
  #endpoint: string | undefined;
  #token: Buffer | undefined;
  #taken = false;
  #forgotten = false;

  #secrets: VrtSecrets;
  constructor(secrets: VrtSecrets) {
    this.#secrets = secrets;
  }

  line(value: string): boolean {
    assert.equal(this.#forgotten, false, "Closed published announcement refused");
    assert.ok(Buffer.byteLength(value) <= 16_384, "Published VRT announcement row too large");
    const line = value.trim();
    if (line === publishedTokenNotice) {
      assert.equal(this.#notice, false, "Repeated published token notice refused");
      this.#notice = true;
    } else if (line.startsWith("VRT endpoint:")) {
      assert.ok(
        this.#notice && !this.#endpoint,
        "Published endpoint order or multiplicity refused",
      );
      let url: URL;
      const endpoint = line.slice("VRT endpoint:".length).trim();
      try {
        url = new URL(endpoint);
      } catch {
        throw new Error("Published loopback endpoint refused");
      }
      assert.ok(
        url.protocol === "http:" &&
          url.hostname === "127.0.0.1" &&
          url.port &&
          endpoint === url.origin,
        "Published loopback endpoint refused",
      );
      this.#endpoint = endpoint;
    } else if (line.startsWith("Session token:")) {
      assert.ok(
        this.#notice && this.#endpoint && !this.#token,
        "Published credential order or multiplicity refused",
      );
      const token = line.slice("Session token:".length).trim();
      this.#secrets.remember(token);
      this.#token = Buffer.from(token);
    }
    return !!(this.#notice && this.#endpoint && this.#token);
  }

  take() {
    assert.ok(
      this.#notice && this.#endpoint && this.#token && !this.#taken,
      "Published VRT announcement incomplete or already taken",
    );
    this.#taken = true;
    return {
      endpoint: this.#endpoint,
      token: this.#token.toString(),
      tokenPrivacyNotice: true as const,
    };
  }

  forget(): void {
    this.#forgotten = true;
    this.#token?.fill(0);
    this.#token = undefined;
    this.#endpoint = undefined;
    this.#taken = true;
  }
}
