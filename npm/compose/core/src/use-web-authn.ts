import { computed, readonly, ref, shallowRef, toValue, unref } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** User verification requirement of a WebAuthn ceremony. */
export type WebAuthnUserVerification = "required" | "preferred" | "discouraged";

/** Attestation conveyance preference of a registration ceremony. */
export type WebAuthnAttestation = "none" | "indirect" | "direct" | "enterprise";

/** Discoverable-credential (resident key) requirement. */
export type WebAuthnResidentKey = "discouraged" | "preferred" | "required";

/** Authenticator attachment modality. */
export type WebAuthnAuthenticatorAttachment = "platform" | "cross-platform";

/** Authenticator transport hint. */
export type WebAuthnTransport = "usb" | "nfc" | "ble" | "hybrid" | "internal";

/** Client hint about the authenticator the relying party expects. */
export type WebAuthnHint = "security-key" | "client-device" | "hybrid";

/** Credential mediation requirement; `"conditional"` powers passkey autofill. */
export type WebAuthnMediation = "silent" | "optional" | "conditional" | "required";

/** Credential descriptor in server JSON form. */
export interface PublicKeyCredentialDescriptorJSON {
  /** Credential type. */
  readonly type: "public-key";
  /** Base64url-encoded credential id. */
  readonly id: string;
  /** Transports the credential is reachable through. */
  readonly transports?: readonly WebAuthnTransport[];
}

/** Credential descriptor in binary form. */
export interface WebAuthnCredentialDescriptor {
  /** Credential type. */
  type: "public-key";
  /** Raw credential id. */
  id: Uint8Array<ArrayBuffer>;
  /** Transports the credential is reachable through. */
  transports?: WebAuthnTransport[];
}

/** Authenticator selection criteria of a registration ceremony. */
export interface WebAuthnAuthenticatorSelection {
  /** Required attachment modality. */
  authenticatorAttachment?: WebAuthnAuthenticatorAttachment;
  /** Discoverable-credential requirement. */
  residentKey?: WebAuthnResidentKey;
  /** Legacy discoverable-credential flag. */
  requireResidentKey?: boolean;
  /** User verification requirement. */
  userVerification?: WebAuthnUserVerification;
}

/** Relying party entity. */
export interface WebAuthnRelyingParty {
  /** Relying party id (a registrable domain). */
  id?: string;
  /** Human-readable name. */
  name: string;
}

/** Public-key algorithm parameter. */
export interface WebAuthnCredentialParameter {
  /** Credential type. */
  type: "public-key";
  /** COSE algorithm identifier (for example -7 for ES256). */
  alg: number;
}

/** `PublicKeyCredentialCreationOptions` as serialized by a server (binary fields base64url). */
export interface PublicKeyCredentialCreationOptionsJSON {
  /** Relying party. */
  readonly rp: Readonly<WebAuthnRelyingParty>;
  /** User account with a base64url-encoded `id`. */
  readonly user: { readonly id: string; readonly name: string; readonly displayName: string };
  /** Base64url-encoded challenge. */
  readonly challenge: string;
  /** Acceptable algorithms in preference order. */
  readonly pubKeyCredParams: readonly Readonly<WebAuthnCredentialParameter>[];
  /** Ceremony timeout in milliseconds. */
  readonly timeout?: number;
  /** Credentials that must not be re-registered. */
  readonly excludeCredentials?: readonly PublicKeyCredentialDescriptorJSON[];
  /** Authenticator selection criteria. */
  readonly authenticatorSelection?: Readonly<WebAuthnAuthenticatorSelection>;
  /** Attestation preference. */
  readonly attestation?: WebAuthnAttestation;
  /** Authenticator hints. */
  readonly hints?: readonly WebAuthnHint[];
  /** Client extension inputs; PRF values are decoded from base64url. */
  readonly extensions?: Readonly<Record<string, unknown>>;
}

/** `PublicKeyCredentialRequestOptions` as serialized by a server (binary fields base64url). */
export interface PublicKeyCredentialRequestOptionsJSON {
  /** Base64url-encoded challenge. */
  readonly challenge: string;
  /** Ceremony timeout in milliseconds. */
  readonly timeout?: number;
  /** Relying party id. */
  readonly rpId?: string;
  /** Allowed credentials; empty or omitted for discoverable credentials. */
  readonly allowCredentials?: readonly PublicKeyCredentialDescriptorJSON[];
  /** User verification requirement. */
  readonly userVerification?: WebAuthnUserVerification;
  /** Authenticator hints. */
  readonly hints?: readonly WebAuthnHint[];
  /** Client extension inputs; PRF values are decoded from base64url. */
  readonly extensions?: Readonly<Record<string, unknown>>;
}

/** Binary creation options passed to `navigator.credentials.create`. */
export interface WebAuthnCreationInit {
  /** Relying party. */
  rp: WebAuthnRelyingParty;
  /** User account. */
  user: { id: Uint8Array<ArrayBuffer>; name: string; displayName: string };
  /** Challenge bytes. */
  challenge: Uint8Array<ArrayBuffer>;
  /** Acceptable algorithms in preference order. */
  pubKeyCredParams: WebAuthnCredentialParameter[];
  /** Ceremony timeout in milliseconds. */
  timeout?: number;
  /** Credentials that must not be re-registered. */
  excludeCredentials?: WebAuthnCredentialDescriptor[];
  /** Authenticator selection criteria. */
  authenticatorSelection?: WebAuthnAuthenticatorSelection;
  /** Attestation preference. */
  attestation?: WebAuthnAttestation;
  /** Authenticator hints. */
  hints?: WebAuthnHint[];
  /** Client extension inputs. */
  extensions?: Record<string, unknown>;
}

/** Binary request options passed to `navigator.credentials.get`. */
export interface WebAuthnRequestInit {
  /** Challenge bytes. */
  challenge: Uint8Array<ArrayBuffer>;
  /** Ceremony timeout in milliseconds. */
  timeout?: number;
  /** Relying party id. */
  rpId?: string;
  /** Allowed credentials. */
  allowCredentials?: WebAuthnCredentialDescriptor[];
  /** User verification requirement. */
  userVerification?: WebAuthnUserVerification;
  /** Authenticator hints. */
  hints?: WebAuthnHint[];
  /** Client extension inputs. */
  extensions?: Record<string, unknown>;
}

/** Fields shared by every returned `PublicKeyCredential`. */
export interface PublicKeyCredentialLike {
  /** Base64url credential id. */
  readonly id: string;
  /** Raw credential id. */
  readonly rawId: ArrayBuffer;
  /** Credential type (`"public-key"`). */
  readonly type: string;
  /** Attachment of the authenticator that produced the credential. */
  readonly authenticatorAttachment?: string | null;
  /** Client extension outputs. */
  getClientExtensionResults(): object;
}

/** Credential returned by a registration ceremony. */
export interface RegistrationCredentialLike extends PublicKeyCredentialLike {
  /** Attestation response. */
  readonly response: {
    /** Client data. */
    readonly clientDataJSON: ArrayBuffer;
    /** CBOR attestation object. */
    readonly attestationObject: ArrayBuffer;
    /** Transports reported by the authenticator. */
    getTransports?(): readonly string[];
    /** DER SubjectPublicKeyInfo of the new credential. */
    getPublicKey?(): ArrayBuffer | null;
    /** COSE algorithm of the new credential. */
    getPublicKeyAlgorithm?(): number;
    /** Authenticator data. */
    getAuthenticatorData?(): ArrayBuffer;
  };
}

/** Credential returned by an authentication ceremony. */
export interface AuthenticationCredentialLike extends PublicKeyCredentialLike {
  /** Assertion response. */
  readonly response: {
    /** Client data. */
    readonly clientDataJSON: ArrayBuffer;
    /** Authenticator data. */
    readonly authenticatorData: ArrayBuffer;
    /** Assertion signature. */
    readonly signature: ArrayBuffer;
    /** User handle of a discoverable credential. */
    readonly userHandle: ArrayBuffer | null;
  };
}

/** JSON-safe registration result, ready to POST to a server. */
export interface RegistrationResponseJSON {
  /** Base64url credential id. */
  readonly id: string;
  /** Base64url raw credential id. */
  readonly rawId: string;
  /** Credential type. */
  readonly type: "public-key";
  /** Authenticator attachment, when known. */
  readonly authenticatorAttachment?: WebAuthnAuthenticatorAttachment;
  /** Client extension outputs. */
  readonly clientExtensionResults: object;
  /** Attestation response with base64url fields. */
  readonly response: {
    /** Base64url client data. */
    readonly clientDataJSON: string;
    /** Base64url attestation object. */
    readonly attestationObject: string;
    /** Known transports. */
    readonly transports?: readonly WebAuthnTransport[];
    /** Base64url public key, when exposed. */
    readonly publicKey?: string;
    /** COSE algorithm, when exposed. */
    readonly publicKeyAlgorithm?: number;
    /** Base64url authenticator data, when exposed. */
    readonly authenticatorData?: string;
  };
}

/** JSON-safe authentication result, ready to POST to a server. */
export interface AuthenticationResponseJSON {
  /** Base64url credential id. */
  readonly id: string;
  /** Base64url raw credential id. */
  readonly rawId: string;
  /** Credential type. */
  readonly type: "public-key";
  /** Authenticator attachment, when known. */
  readonly authenticatorAttachment?: WebAuthnAuthenticatorAttachment;
  /** Client extension outputs. */
  readonly clientExtensionResults: object;
  /** Assertion response with base64url fields. */
  readonly response: {
    /** Base64url client data. */
    readonly clientDataJSON: string;
    /** Base64url authenticator data. */
    readonly authenticatorData: string;
    /** Base64url signature. */
    readonly signature: string;
    /** Base64url user handle, when present. */
    readonly userHandle?: string;
  };
}

/** Minimal `navigator.credentials` consumed by {@link useWebAuthn}. */
export interface WebAuthnCredentialsHost {
  /** Run a registration ceremony. */
  create(options: {
    publicKey: WebAuthnCreationInit;
    signal?: AbortSignal;
    mediation?: WebAuthnMediation;
  }): Promise<unknown>;
  /** Run an authentication ceremony. */
  get(options: {
    publicKey: WebAuthnRequestInit;
    signal?: AbortSignal;
    mediation?: WebAuthnMediation;
  }): Promise<unknown>;
}

/** Static capability checks of `PublicKeyCredential`. */
export interface PublicKeyCredentialStatics {
  /** Whether a user-verifying platform authenticator (Touch ID, Windows Hello) exists. */
  isUserVerifyingPlatformAuthenticatorAvailable?(): Promise<boolean>;
  /** Whether conditional mediation (passkey autofill) is available. */
  isConditionalMediationAvailable?(): Promise<boolean>;
}

/** Options for {@link useWebAuthn}. */
export interface UseWebAuthnOptions {
  /**
   * Credentials container for alternate runtimes and tests.
   *
   * @default window.navigator.credentials when `window.PublicKeyCredential` exists
   */
  readonly credentials?: MaybeRefOrGetter<WebAuthnCredentialsHost | null | undefined>;

  /**
   * `PublicKeyCredential` interface used for capability checks. A ref (not a
   * getter) because the browser value is a constructor function.
   *
   * @default window.PublicKeyCredential
   */
  readonly publicKeyCredential?: MaybeRef<PublicKeyCredentialStatics | null | undefined>;
}

/** Per-ceremony options of {@link WebAuthnControls.create} and {@link WebAuthnControls.get}. */
export interface WebAuthnCeremonyOptions {
  /**
   * Mediation requirement; `"conditional"` for passkey autofill.
   *
   * @default undefined
   */
  readonly mediation?: WebAuthnMediation;

  /**
   * Additional signal aborting the ceremony.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;
}

/** Discriminated outcome of a WebAuthn ceremony. */
export type WebAuthnResult<Credential> =
  | {
      /** The ceremony produced a credential. */
      readonly status: "success";
      /** Returned credential. */
      readonly credential: Credential;
    }
  | {
      /**
       * `aborted`: aborted by `abort()`, a newer ceremony, or `signal`;
       * `not-allowed`: the user cancelled or the ceremony timed out;
       * `unsupported`: WebAuthn is unavailable; `failed`: anything else.
       */
      readonly status: "aborted" | "not-allowed" | "unsupported" | "failed";
      /** Error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link useWebAuthn}. */
export interface WebAuthnControls {
  /** Whether WebAuthn is available. */
  readonly supported: ComputedRef<boolean>;
  /** Whether a ceremony is in progress. */
  readonly pending: Readonly<Ref<boolean>>;
  /** Most recent ceremony failure (not aborts), cleared when a ceremony starts. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Register a credential. Aborts any pending ceremony first.
   *
   * @param options Binary options or server JSON (parsed automatically).
   * @param ceremony Mediation and extra abort signal.
   * @returns The discriminated outcome; never rejects.
   */
  readonly create: (
    options: WebAuthnCreationInit | PublicKeyCredentialCreationOptionsJSON,
    ceremony?: WebAuthnCeremonyOptions,
  ) => Promise<WebAuthnResult<RegistrationCredentialLike>>;
  /**
   * Authenticate with a credential. Aborts any pending ceremony first.
   *
   * @param options Binary options or server JSON (parsed automatically).
   * @param ceremony Mediation (for example `"conditional"`) and extra abort signal.
   * @returns The discriminated outcome; never rejects.
   */
  readonly get: (
    options: WebAuthnRequestInit | PublicKeyCredentialRequestOptionsJSON,
    ceremony?: WebAuthnCeremonyOptions,
  ) => Promise<WebAuthnResult<AuthenticationCredentialLike>>;
  /**
   * Abort the pending ceremony, if any.
   *
   * @param reason Abort reason.
   */
  readonly abort: (reason?: unknown) => void;
  /**
   * Whether a user-verifying platform authenticator exists.
   *
   * @returns `false` when unsupported or the check fails.
   */
  readonly isUserVerifyingPlatformAuthenticatorAvailable: () => Promise<boolean>;
  /**
   * Whether conditional mediation (passkey autofill) is available.
   *
   * @returns `false` when unsupported or the check fails.
   */
  readonly isConditionalMediationAvailable: () => Promise<boolean>;
}

const transports: readonly string[] = ["usb", "nfc", "ble", "hybrid", "internal"];

function isTransport(value: string): value is WebAuthnTransport {
  return transports.includes(value);
}

function isAttachment(value: unknown): value is WebAuthnAuthenticatorAttachment {
  return value === "platform" || value === "cross-platform";
}

/**
 * Encode bytes as unpadded base64url.
 *
 * @param bytes Buffer or view to encode.
 * @returns Base64url text without `=` padding.
 */
export function encodeBase64Url(bytes: ArrayBuffer | ArrayBufferView): string {
  const view =
    bytes instanceof ArrayBuffer
      ? new Uint8Array(bytes)
      : new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let binary = "";
  for (const byte of view) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/u, "");
}

/**
 * Decode base64url (padded or unpadded) into bytes.
 *
 * @param text Base64url text.
 * @throws `TypeError` tagged `VIZE_COMPOSE_WEBAUTHN_INVALID_BASE64URL` for malformed input.
 * @returns Decoded bytes.
 */
export function decodeBase64Url(text: string): Uint8Array<ArrayBuffer> {
  const body = text.replace(/=+$/u, "");
  if (!/^[\w-]*$/u.test(body) || body.length % 4 === 1) {
    throw new TypeError(`[VIZE_COMPOSE_WEBAUTHN_INVALID_BASE64URL] invalid base64url input`);
  }
  const binary = atob(body.replaceAll("-", "+").replaceAll("_", "/"));
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return bytes;
}

function parseDescriptors(
  list: readonly PublicKeyCredentialDescriptorJSON[],
): WebAuthnCredentialDescriptor[] {
  return list.map((descriptor) => ({
    type: descriptor.type,
    id: decodeBase64Url(descriptor.id),
    ...(descriptor.transports ? { transports: [...descriptor.transports] } : {}),
  }));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parsePrfValues(value: unknown): unknown {
  if (!isRecord(value)) return value;
  return {
    ...value,
    ...(typeof value.first === "string" ? { first: decodeBase64Url(value.first) } : {}),
    ...(typeof value.second === "string" ? { second: decodeBase64Url(value.second) } : {}),
  };
}

function parseExtensions(extensions: Readonly<Record<string, unknown>>): Record<string, unknown> {
  const prf = extensions.prf;
  if (!isRecord(prf)) return { ...extensions };
  const byCredential = prf.evalByCredential;
  return {
    ...extensions,
    prf: {
      ...prf,
      ...(prf.eval === undefined ? {} : { eval: parsePrfValues(prf.eval) }),
      ...(isRecord(byCredential)
        ? {
            evalByCredential: Object.fromEntries(
              Object.entries(byCredential).map(([id, values]) => [id, parsePrfValues(values)]),
            ),
          }
        : {}),
    },
  };
}

function serializeExtensionValue(value: unknown): unknown {
  if (value instanceof ArrayBuffer || ArrayBuffer.isView(value)) return encodeBase64Url(value);
  if (Array.isArray(value)) return value.map(serializeExtensionValue);
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => [key, serializeExtensionValue(entry)]),
    );
  }
  return value;
}

function serializeExtensions(results: object): object {
  return serializeExtensionValue(results) as object;
}

/**
 * Convert server creation-options JSON into binary options for `create`.
 *
 * @param json Options with base64url `challenge`, `user.id`, and credential ids.
 * @throws `TypeError` tagged `VIZE_COMPOSE_WEBAUTHN_INVALID_BASE64URL` for malformed fields.
 * @returns Binary creation options.
 */
export function parseCreationOptionsFromJSON(
  json: PublicKeyCredentialCreationOptionsJSON,
): WebAuthnCreationInit {
  const { excludeCredentials, authenticatorSelection, hints, extensions, ...rest } = json;
  return {
    ...rest,
    rp: { ...json.rp },
    user: { ...json.user, id: decodeBase64Url(json.user.id) },
    challenge: decodeBase64Url(json.challenge),
    pubKeyCredParams: json.pubKeyCredParams.map((parameter) => ({ ...parameter })),
    ...(excludeCredentials ? { excludeCredentials: parseDescriptors(excludeCredentials) } : {}),
    ...(authenticatorSelection ? { authenticatorSelection: { ...authenticatorSelection } } : {}),
    ...(hints ? { hints: [...hints] } : {}),
    ...(extensions ? { extensions: parseExtensions(extensions) } : {}),
  };
}

/**
 * Convert server request-options JSON into binary options for `get`.
 *
 * @param json Options with base64url `challenge` and credential ids.
 * @throws `TypeError` tagged `VIZE_COMPOSE_WEBAUTHN_INVALID_BASE64URL` for malformed fields.
 * @returns Binary request options.
 */
export function parseRequestOptionsFromJSON(
  json: PublicKeyCredentialRequestOptionsJSON,
): WebAuthnRequestInit {
  const { allowCredentials, hints, extensions, ...rest } = json;
  return {
    ...rest,
    challenge: decodeBase64Url(json.challenge),
    ...(allowCredentials ? { allowCredentials: parseDescriptors(allowCredentials) } : {}),
    ...(hints ? { hints: [...hints] } : {}),
    ...(extensions ? { extensions: parseExtensions(extensions) } : {}),
  };
}

/**
 * Serialize a registration credential into JSON-safe base64url fields.
 *
 * @param credential Credential resolved by `create`.
 * @returns Server-ready registration response.
 */
export function serializeRegistrationCredential(
  credential: RegistrationCredentialLike,
): RegistrationResponseJSON {
  const { response } = credential;
  const publicKey = response.getPublicKey?.() ?? null;
  const authenticatorData = response.getAuthenticatorData?.();
  const algorithm = response.getPublicKeyAlgorithm?.();
  const known = response.getTransports?.().filter(isTransport);
  return {
    id: credential.id,
    rawId: encodeBase64Url(credential.rawId),
    type: "public-key",
    ...(isAttachment(credential.authenticatorAttachment)
      ? { authenticatorAttachment: credential.authenticatorAttachment }
      : {}),
    clientExtensionResults: serializeExtensions(credential.getClientExtensionResults()),
    response: {
      clientDataJSON: encodeBase64Url(response.clientDataJSON),
      attestationObject: encodeBase64Url(response.attestationObject),
      ...(known ? { transports: known } : {}),
      ...(publicKey ? { publicKey: encodeBase64Url(publicKey) } : {}),
      ...(algorithm === undefined ? {} : { publicKeyAlgorithm: algorithm }),
      ...(authenticatorData ? { authenticatorData: encodeBase64Url(authenticatorData) } : {}),
    },
  };
}

/**
 * Serialize an authentication credential into JSON-safe base64url fields.
 *
 * @param credential Credential resolved by `get`.
 * @returns Server-ready authentication response.
 */
export function serializeAuthenticationCredential(
  credential: AuthenticationCredentialLike,
): AuthenticationResponseJSON {
  const { response } = credential;
  return {
    id: credential.id,
    rawId: encodeBase64Url(credential.rawId),
    type: "public-key",
    ...(isAttachment(credential.authenticatorAttachment)
      ? { authenticatorAttachment: credential.authenticatorAttachment }
      : {}),
    clientExtensionResults: serializeExtensions(credential.getClientExtensionResults()),
    response: {
      clientDataJSON: encodeBase64Url(response.clientDataJSON),
      authenticatorData: encodeBase64Url(response.authenticatorData),
      signature: encodeBase64Url(response.signature),
      ...(response.userHandle ? { userHandle: encodeBase64Url(response.userHandle) } : {}),
    },
  };
}

function hasResponseField(value: unknown, field: string): boolean {
  if (typeof value !== "object" || value === null || !("response" in value)) return false;
  const { response } = value;
  return typeof response === "object" && response !== null && field in response;
}

function isRegistrationCredential(value: unknown): value is RegistrationCredentialLike {
  return hasResponseField(value, "attestationObject");
}

function isAuthenticationCredential(value: unknown): value is AuthenticationCredentialLike {
  return hasResponseField(value, "signature");
}

function isStatics(candidate: unknown): candidate is PublicKeyCredentialStatics {
  return typeof candidate === "function";
}

function browserStatics(): PublicKeyCredentialStatics | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "PublicKeyCredential");
  return isStatics(candidate) ? candidate : undefined;
}

function browserCredentials(): WebAuthnCredentialsHost | undefined {
  if (browserStatics() === undefined) return undefined;
  const container = window.navigator.credentials;
  if (!container) return undefined;
  return {
    create: (options) => container.create(options),
    get: (options) => container.get(options),
  };
}

function isJSONOptions<Json extends { readonly challenge: string }>(
  input: Json | { readonly challenge: Uint8Array },
): input is Json {
  return typeof input.challenge === "string";
}

function errorName(error: unknown): unknown {
  return typeof error === "object" && error !== null && "name" in error ? error.name : undefined;
}

/**
 * Register and authenticate passkeys with the Web Authentication API.
 *
 * `create` and `get` accept binary options or server JSON, run one ceremony
 * at a time (starting one aborts the previous), and resolve to a
 * discriminated {@link WebAuthnResult}. Use the exported pure helpers to
 * serialize results for the server. A pending ceremony is aborted when the
 * owning reactive scope stops; outside a scope call `abort()`.
 *
 * Server rendering: nothing is requested, `supported` and `pending` are false.
 *
 * @example
 * ```ts
 * const webAuthn = useWebAuthn();
 * const result = await webAuthn.get(await fetchJSON("/login/options"));
 * if (result.status === "success") await post(serializeAuthenticationCredential(result.credential));
 * ```
 *
 * @param options Capability hosts.
 * @default options {}
 * @returns Ceremony state and actions.
 */
export function useWebAuthn(options: UseWebAuthnOptions = {}): WebAuthnControls {
  const pending = ref(false);
  const error = shallowRef<unknown>(undefined);
  let controller: AbortController | undefined;

  const resolveCredentials = (): WebAuthnCredentialsHost | undefined =>
    options.credentials === undefined
      ? browserCredentials()
      : (toValue(options.credentials) ?? undefined);
  const resolveStatics = (): PublicKeyCredentialStatics | undefined =>
    options.publicKeyCredential === undefined
      ? browserStatics()
      : (unref(options.publicKeyCredential) ?? undefined);

  const abort = (reason?: unknown): void => {
    const current = controller;
    controller = undefined;
    pending.value = false;
    current?.abort(reason);
  };

  const run = async <Credential>(
    ceremony: WebAuthnCeremonyOptions,
    invoke: (
      host: WebAuthnCredentialsHost,
      request: { signal: AbortSignal; mediation?: WebAuthnMediation },
    ) => Promise<unknown>,
    guard: (value: unknown) => value is Credential,
  ): Promise<WebAuthnResult<Credential>> => {
    const host = resolveCredentials();
    if (!host) return { status: "unsupported", error: undefined };
    abort();
    const own = new AbortController();
    controller = own;
    const external = ceremony.signal;
    const forward = (): void => own.abort(external?.reason);
    if (external?.aborted) forward();
    else external?.addEventListener("abort", forward, { once: true });
    pending.value = true;
    error.value = undefined;
    try {
      const value = await invoke(host, {
        signal: own.signal,
        ...(ceremony.mediation ? { mediation: ceremony.mediation } : {}),
      });
      if (own.signal.aborted) return { status: "aborted", error: own.signal.reason };
      if (!guard(value)) throw new TypeError("[VIZE_COMPOSE_WEBAUTHN_NO_CREDENTIAL] no credential");
      return { status: "success", credential: value };
    } catch (cause) {
      if (own.signal.aborted || errorName(cause) === "AbortError") {
        return { status: "aborted", error: cause };
      }
      error.value = cause;
      return {
        status: errorName(cause) === "NotAllowedError" ? "not-allowed" : "failed",
        error: cause,
      };
    } finally {
      external?.removeEventListener("abort", forward);
      if (controller === own) {
        controller = undefined;
        pending.value = false;
      }
    }
  };

  const create: WebAuthnControls["create"] = (input, ceremony = {}) =>
    run(
      ceremony,
      (host, request) =>
        host.create({
          ...request,
          publicKey: isJSONOptions(input) ? parseCreationOptionsFromJSON(input) : input,
        }),
      isRegistrationCredential,
    );

  const get: WebAuthnControls["get"] = (input, ceremony = {}) =>
    run(
      ceremony,
      (host, request) =>
        host.get({
          ...request,
          publicKey: isJSONOptions(input) ? parseRequestOptionsFromJSON(input) : input,
        }),
      isAuthenticationCredential,
    );

  const check = async (key: keyof PublicKeyCredentialStatics): Promise<boolean> => {
    const statics = resolveStatics();
    try {
      return (await statics?.[key]?.call(statics)) === true;
    } catch {
      return false;
    }
  };

  tryOnScopeDispose(() => abort());

  return {
    supported: computed(() => resolveCredentials() !== undefined),
    pending: readonly(pending),
    error: readonly(error),
    create,
    get,
    abort,
    isUserVerifyingPlatformAuthenticatorAvailable: () =>
      check("isUserVerifyingPlatformAuthenticatorAvailable"),
    isConditionalMediationAvailable: () => check("isConditionalMediationAvailable"),
  };
}
