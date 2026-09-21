# Extension contracts — versioning and compatibility policy

> [!NOTE]
> The written compatibility policy charter #15's extension contracts carry
> from day one (the Swift macro lesson), committed before the contracts GA
> (P6-8). It governs the `vize:contracts` WIT package in `contracts/wit/`
> and every payload that crosses it. Vize-internal formats stay free until
> GA under charter #23; this policy is what GA freezes.

## What is versioned

A contract release is four facts moving together, recorded as one canonical
**contract surface** (`vize_marquette::ContractSurface`) per released version
in `contracts/versions/<package>@<version>.json`:

| Fact                 | Where it lives                                          | Who checks it                   |
| -------------------- | ------------------------------------------------------- | ------------------------------- |
| Package version      | `package vize:contracts@X.Y.Z;` in the WIT              | component linking; this policy  |
| Protocol version     | the `handshake.capability` integer                      | the host, before any other call |
| Page schema versions | `page.schema-version` (`s1-page`, `s2-page`)            | the host, before parsing a page |
| Required features    | the features each world's guest must offer in handshake | the host, at negotiation        |

The surface lists every interface, named type (members in order: the
canonical ABI is positional), function signature and world import/export,
with named types qualified by their interface and `use` aliases transparent.
Its canonical serialization is deterministic and fingerprinted, as marquette
application contracts are.

## Additive and breaking

Compatibility is judged from the guest's side: a change is **additive** when
every guest built against the older version keeps working with a host at the
newer version, and **breaking** otherwise. Where the direction of a value is
not modelled, a change to an existing type is breaking — the classifier
(`vize_marquette::compare_surfaces`) may over-report, never under-report.

| Change                                                          | Class    |
| --------------------------------------------------------------- | -------- |
| New interface, world, named type or page kind                   | additive |
| New function on an interface only the host implements (imports) | additive |
| New world import (the host provides more)                       | additive |
| A required feature dropped (the host requires less)             | additive |
| New function on an interface a guest implements (exports)       | breaking |
| New or removed world export                                     | breaking |
| Removed world import, interface, type, function, page kind      | breaking |
| Any change to a record's fields, a variant/enum's cases, flags  | breaking |
| Any change to a function's parameters or result                 | breaking |
| A page schema version changed                                   | breaking |
| A required feature added                                        | breaking |
| The protocol version or the package identity changed            | breaking |

Features a host does not know are set aside, never refused: offering more is
always additive for a guest.

## How versions move

`vize_marquette::check_version_policy` enforces, for the classified change
from the previous release:

- **Any change moves the package version up**; it never decreases.
- **Before 1.0**, the minor number is the breaking axis: breaking needs at
  least `0.(Y+1).0`, additive any increase (`0.Y.(Z+1)`).
- **From 1.0**, breaking needs `(X+1).0.0`, additive `X.(Y+1).0`.
- **A breaking change raises the protocol version by exactly one**, so a host
  refuses an incompatible guest at the handshake with its exact message,
  before the first call.

A page schema bump is breaking because the host reads exactly one version of
each page; accepting two versions at once would make the bump additive, and is
a deliberate host change, not a relaxation of this rule.

## The conformance suite moves with the contracts

TS-48's goldens and guests are pinned to the package version they exercise:
`crates/vize_extension_host/tests/contract_surface.rs` holds every released
surface to this policy pairwise, and holds the newest one to what
`contracts/wit/` and the host's handshake constants describe today, byte for
byte. Changing the WIT therefore means choosing the version this policy
requires and committing its surface (`VIZE_CONTRACT_SURFACE_BLESS=1` writes it
for review) in the same change as the TS-48 golden updates. The classifier
itself is pinned by `crates/vize_marquette/tests/contract_compat.rs` (a
deliberately breaking WIT change flagged and its additive-sized bump refused;
a purely additive one not flagged) and `tests/contract_version.rs`.

## Deprecation and GA

Until the contracts GA (phase 6 exit), `0.x` releases may break under the
rules above; each breaking release names its guest-visible changes in the
release notes. From GA, a breaking release is a major version, and the host
will keep serving the previous major's protocol version for at least one minor
release cycle before refusing it.
