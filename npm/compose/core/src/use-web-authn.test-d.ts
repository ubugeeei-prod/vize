/** Compile-only assertions for the `use-web-authn` type contracts. */

import {
  decodeBase64Url,
  parseCreationOptionsFromJSON,
  serializeAuthenticationCredential,
  useWebAuthn,
} from "./use-web-authn.ts";
import type {
  AuthenticationCredentialLike,
  AuthenticationResponseJSON,
  PublicKeyCredentialCreationOptionsJSON,
  RegistrationCredentialLike,
  WebAuthnCreationInit,
  WebAuthnCredentialsHost,
  WebAuthnResult,
} from "./use-web-authn.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const webAuthn = useWebAuthn();
declare const json: PublicKeyCredentialCreationOptionsJSON;

type _CreateResult = Expect<
  Equal<Awaited<ReturnType<typeof webAuthn.create>>, WebAuthnResult<RegistrationCredentialLike>>
>;
type _GetResult = Expect<
  Equal<Awaited<ReturnType<typeof webAuthn.get>>, WebAuthnResult<AuthenticationCredentialLike>>
>;
type _Parsed = Expect<Equal<ReturnType<typeof parseCreationOptionsFromJSON>, WebAuthnCreationInit>>;
type _Decoded = Expect<Equal<ReturnType<typeof decodeBase64Url>, Uint8Array<ArrayBuffer>>>;
type _Serialized = Expect<
  Equal<ReturnType<typeof serializeAuthenticationCredential>, AuthenticationResponseJSON>
>;

async function narrow(): Promise<void> {
  const result = await webAuthn.get({ challenge: "AQ" });
  if (result.status === "success") {
    type _Credential = Expect<Equal<typeof result.credential, AuthenticationCredentialLike>>;
  }
}
void narrow;

declare const credentials: CredentialsContainer;
credentials satisfies WebAuthnCredentialsHost;

// @ts-expect-error userVerification is a closed union.
void webAuthn.get({ challenge: "AQ", userVerification: "sometimes" });

// @ts-expect-error residentKey is a closed union.
void webAuthn.create({ ...json, authenticatorSelection: { residentKey: "maybe" } });

// @ts-expect-error mediation is a closed union.
void webAuthn.get({ challenge: "AQ" }, { mediation: "automatic" });

// @ts-expect-error pending is read-only.
webAuthn.pending.value = true;
