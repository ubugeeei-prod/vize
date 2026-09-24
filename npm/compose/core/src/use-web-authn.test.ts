import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import {
  decodeBase64Url,
  encodeBase64Url,
  parseCreationOptionsFromJSON,
  parseRequestOptionsFromJSON,
  serializeAuthenticationCredential,
  serializeRegistrationCredential,
  useWebAuthn,
} from "./use-web-authn.ts";
import type {
  AuthenticationCredentialLike,
  PublicKeyCredentialCreationOptionsJSON,
  RegistrationCredentialLike,
  WebAuthnCredentialsHost,
  WebAuthnCreationInit,
  WebAuthnMediation,
  WebAuthnRequestInit,
} from "./use-web-authn.ts";

const bytes = (...values: number[]): ArrayBuffer => new Uint8Array(values).buffer;

const registration: RegistrationCredentialLike = {
  id: "AQI",
  rawId: bytes(1, 2),
  type: "public-key",
  authenticatorAttachment: "platform",
  getClientExtensionResults: () => ({ credProps: { rk: true } }),
  response: {
    clientDataJSON: bytes(3),
    attestationObject: bytes(4, 5),
    getTransports: () => ["internal", "bogus", "hybrid"],
    getPublicKey: () => bytes(6),
    getPublicKeyAlgorithm: () => -7,
  },
};

const assertion: AuthenticationCredentialLike = {
  id: "AQI",
  rawId: bytes(1, 2),
  type: "public-key",
  authenticatorAttachment: null,
  getClientExtensionResults: () => ({}),
  response: {
    clientDataJSON: bytes(3),
    authenticatorData: bytes(7),
    signature: bytes(8, 9),
    userHandle: null,
  },
};

const creationJSON: PublicKeyCredentialCreationOptionsJSON = {
  rp: { name: "Vize", id: "vize.dev" },
  user: { id: "AQID", name: "ada", displayName: "Ada" },
  challenge: "_-8",
  pubKeyCredParams: [{ type: "public-key", alg: -7 }],
  excludeCredentials: [{ type: "public-key", id: "AQI", transports: ["usb"] }],
  authenticatorSelection: { residentKey: "required", userVerification: "preferred" },
  attestation: "none",
};

interface Call {
  readonly kind: "create" | "get";
  readonly publicKey: WebAuthnCreationInit | WebAuthnRequestInit;
  readonly signal: AbortSignal | undefined;
  readonly mediation: WebAuthnMediation | undefined;
}

function createHost(result: () => Promise<unknown>): {
  host: WebAuthnCredentialsHost;
  calls: Call[];
} {
  const calls: Call[] = [];
  const host: WebAuthnCredentialsHost = {
    create: (options) => {
      calls.push({
        kind: "create",
        ...options,
        signal: options.signal,
        mediation: options.mediation,
      });
      return result();
    },
    get: (options) => {
      calls.push({ kind: "get", ...options, signal: options.signal, mediation: options.mediation });
      return result();
    },
  };
  return { host, calls };
}

function abortable(signal: AbortSignal | undefined): Promise<unknown> {
  return new Promise((_resolve, reject) => {
    signal?.addEventListener("abort", () => reject(signal.reason));
  });
}

void test("base64url codecs round-trip and reject malformed input", () => {
  const data = new Uint8Array([0, 251, 255, 16, 63]);
  const text = encodeBase64Url(data);
  assert.equal(text, "APv_ED8");
  assert.deepEqual([...decodeBase64Url(text)], [...data]);
  assert.deepEqual([...decodeBase64Url("APv_ED8=")], [...data]);
  assert.equal(encodeBase64Url(new DataView(data.buffer, 1, 2)), "-_8");
  assert.equal(encodeBase64Url(new ArrayBuffer(0)), "");
  assert.throws(() => decodeBase64Url("a+b"), /VIZE_COMPOSE_WEBAUTHN_INVALID_BASE64URL/u);
  assert.throws(() => decodeBase64Url("abcde"), TypeError);
});

void test("parses server JSON into binary options", () => {
  const creation = parseCreationOptionsFromJSON(creationJSON);
  assert.deepEqual([...creation.challenge], [255, 239]);
  assert.deepEqual([...creation.user.id], [1, 2, 3]);
  assert.equal(creation.user.displayName, "Ada");
  assert.deepEqual([...(creation.excludeCredentials?.[0]?.id ?? [])], [1, 2]);
  assert.deepEqual(creation.excludeCredentials?.[0]?.transports, ["usb"]);
  assert.equal(creation.authenticatorSelection?.residentKey, "required");
  assert.equal("hints" in creation, false);

  const request = parseRequestOptionsFromJSON({
    challenge: "AQ",
    rpId: "vize.dev",
    allowCredentials: [{ type: "public-key", id: "Ag" }],
    userVerification: "required",
  });
  assert.deepEqual([...request.challenge], [1]);
  assert.deepEqual([...(request.allowCredentials?.[0]?.id ?? [])], [2]);
  assert.equal("transports" in (request.allowCredentials?.[0] ?? {}), false);
  assert.equal(request.userVerification, "required");
});

void test("decodes PRF extension inputs for registration and authentication", () => {
  const creation = parseCreationOptionsFromJSON({
    ...creationJSON,
    extensions: { prf: { eval: { first: "AQI", second: "Aw" } } },
  });
  const creationPrf = creation.extensions?.prf as {
    eval: { first: Uint8Array; second: Uint8Array };
  };
  assert.deepEqual([...creationPrf.eval.first], [1, 2]);
  assert.deepEqual([...creationPrf.eval.second], [3]);

  const request = parseRequestOptionsFromJSON({
    challenge: "AQ",
    extensions: {
      prf: {
        evalByCredential: { AQI: { first: "BAU", second: "Bg" } },
      },
    },
  });
  const requestPrf = request.extensions?.prf as {
    evalByCredential: Record<string, { first: Uint8Array; second: Uint8Array }>;
  };
  assert.deepEqual([...requestPrf.evalByCredential.AQI!.first], [4, 5]);
  assert.deepEqual([...requestPrf.evalByCredential.AQI!.second], [6]);
});

void test("serializes credentials into JSON-safe objects", () => {
  assert.deepEqual(serializeRegistrationCredential(registration), {
    id: "AQI",
    rawId: "AQI",
    type: "public-key",
    authenticatorAttachment: "platform",
    clientExtensionResults: { credProps: { rk: true } },
    response: {
      clientDataJSON: "Aw",
      attestationObject: "BAU",
      transports: ["internal", "hybrid"],
      publicKey: "Bg",
      publicKeyAlgorithm: -7,
    },
  });
  assert.deepEqual(serializeAuthenticationCredential(assertion), {
    id: "AQI",
    rawId: "AQI",
    type: "public-key",
    clientExtensionResults: {},
    response: { clientDataJSON: "Aw", authenticatorData: "Bw", signature: "CAk" },
  });
});

void test("serializes nested binary extension outputs as base64url", () => {
  const withPrf: AuthenticationCredentialLike = {
    ...assertion,
    getClientExtensionResults: () => ({
      prf: { results: { first: bytes(1, 2), second: new Uint8Array([3, 4]) } },
    }),
  };
  const json = serializeAuthenticationCredential(withPrf);
  assert.deepEqual(json.clientExtensionResults, {
    prf: { results: { first: "AQI", second: "AwQ" } },
  });
  assert.deepEqual(JSON.parse(JSON.stringify(json)), json);
});

void test("create parses JSON options and resolves the credential", async () => {
  const { host, calls } = createHost(() => Promise.resolve(registration));
  const webAuthn = useWebAuthn({ credentials: host });
  assert.equal(webAuthn.supported.value, true);

  const pending = webAuthn.create(creationJSON);
  assert.equal(webAuthn.pending.value, true);
  const result = await pending;
  assert.deepEqual(result, { status: "success", credential: registration });
  assert.equal(webAuthn.pending.value, false);
  assert.equal(calls[0]?.kind, "create");
  assert.ok(calls[0]?.publicKey.challenge instanceof Uint8Array);
  assert.ok(calls[0]?.signal instanceof AbortSignal);
});

void test("get forwards conditional mediation and binary options", async () => {
  const { host, calls } = createHost(() => Promise.resolve(assertion));
  const webAuthn = useWebAuthn({ credentials: host });
  const challenge = new Uint8Array([1]);
  const result = await webAuthn.get({ challenge }, { mediation: "conditional" });
  assert.equal(result.status, "success");
  assert.equal(calls[0]?.publicKey.challenge, challenge);
  assert.equal(calls[0]?.mediation, "conditional");
});

void test("a new ceremony aborts the pending one", async () => {
  let round = 0;
  const { host, calls } = createHost(() => {
    round += 1;
    return round === 1 ? abortable(calls[0]?.signal) : Promise.resolve(assertion);
  });
  const webAuthn = useWebAuthn({ credentials: host });
  const first = webAuthn.get({ challenge: new Uint8Array([1]) }, { mediation: "conditional" });
  const second = webAuthn.get({ challenge: new Uint8Array([2]) });
  assert.equal((await first).status, "aborted");
  assert.equal((await second).status, "success");
  assert.equal(calls[0]?.signal?.aborted, true);
  assert.equal(webAuthn.pending.value, false);
  assert.equal(webAuthn.error.value, undefined);
});

void test("abort(), external signals, and scope disposal cancel ceremonies", async () => {
  const { host, calls } = createHost(() => abortable(calls.at(-1)?.signal));
  const scope = effectScope();
  const webAuthn = scope.run(() => useWebAuthn({ credentials: host }));
  assert.ok(webAuthn);

  const manual = webAuthn.get({ challenge: new Uint8Array([1]) });
  webAuthn.abort("stop");
  assert.deepEqual(await manual, { status: "aborted", error: "stop" });

  const external = new AbortController();
  const viaSignal = webAuthn.get({ challenge: new Uint8Array([1]) }, { signal: external.signal });
  external.abort("external");
  assert.deepEqual(await viaSignal, { status: "aborted", error: "external" });

  const disposed = webAuthn.get({ challenge: new Uint8Array([1]) });
  scope.stop();
  assert.equal((await disposed).status, "aborted");
  assert.equal(webAuthn.pending.value, false);
});

void test("classifies failures and exposes them through error", async () => {
  const denial = new DOMException("cancelled", "NotAllowedError");
  let next: unknown = denial;
  const { host } = createHost(() => (next === null ? Promise.resolve(null) : Promise.reject(next)));
  const webAuthn = useWebAuthn({ credentials: host });

  assert.deepEqual(await webAuthn.get({ challenge: new Uint8Array() }), {
    status: "not-allowed",
    error: denial,
  });
  assert.equal(webAuthn.error.value, denial);

  next = null;
  const empty = await webAuthn.create(creationJSON);
  assert.equal(empty.status, "failed");
  assert.match(String(webAuthn.error.value), /VIZE_COMPOSE_WEBAUTHN_NO_CREDENTIAL/u);

  const invalid = await webAuthn.create({ ...creationJSON, challenge: "@@" });
  assert.equal(invalid.status, "failed");
});

void test("reports unsupported and capability checks without hosts", async () => {
  const webAuthn = useWebAuthn({ credentials: null, publicKeyCredential: null });
  assert.equal(webAuthn.supported.value, false);
  assert.deepEqual(await webAuthn.create(creationJSON), {
    status: "unsupported",
    error: undefined,
  });
  assert.equal(await webAuthn.isConditionalMediationAvailable(), false);
});

void test("capability checks call the PublicKeyCredential statics", async () => {
  class FakePublicKeyCredential {
    static isUserVerifyingPlatformAuthenticatorAvailable(): Promise<boolean> {
      return Promise.resolve(true);
    }

    static isConditionalMediationAvailable(): Promise<boolean> {
      return Promise.reject(new Error("boom"));
    }
  }
  const webAuthn = useWebAuthn({ publicKeyCredential: FakePublicKeyCredential });
  assert.equal(await webAuthn.isUserVerifyingPlatformAuthenticatorAvailable(), true);
  assert.equal(await webAuthn.isConditionalMediationAvailable(), false);
});

void test("server rendering starts no ceremony", async () => {
  const state = await renderComposableOnServer(() => {
    const webAuthn = useWebAuthn();
    return { supported: webAuthn.supported, pending: webAuthn.pending, error: webAuthn.error };
  });
  assert.equal(state, '{"supported":false,"pending":false}');
});
