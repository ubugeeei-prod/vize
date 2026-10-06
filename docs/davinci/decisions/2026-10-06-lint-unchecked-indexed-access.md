# Native lint indexed-access options

Issue: [#7995](https://github.com/ubugeeei-prod/vize/issues/7995).
Paired [source decision](https://github.com/ubugeeei-prod/vize/issues/7995#issuecomment-6008612958).
Reporter: `ubugeeei` / `71201308`.

## Cause and bounded change

Patina creates its real Corsa project from an existing compiler-option overlay.
Canon's effective snapshot includes authored `noUncheckedIndexedAccess`, but
the overlay previously copied only `paths`, `baseUrl`, `types`, and `typeRoots`.
The native checker consequently sees ordinary object/string indexed-access
types instead of their authored nullable types. Copy the one additional option
through that same overlay; keep the actual condition-node queries, classifier,
strict defaults, option allowances, session lifecycle, and stable schemas.

Absent options add no field. Explicit false remains false. Canon's existing
effective loader owns `extends`, explicit overrides, and nearest-file lookup.
There is no new parse, AST walk, pipeline stage, protocol or dependency.

## Whole original custody

The independent corpus is
`tests/_fixtures/differential/linter/unchecked-index-access`. It retains the
complete issue body and all four fenced blocks; extracted fenced carriers use
an explicit trailing LF. The original invocation remains
`lint -f plain --help-level none src/MyTouch.vue`.

| Original         | Bytes | SHA256                                                             |
| ---------------- | ----: | ------------------------------------------------------------------ |
| Whole issue body |  2776 | `32f908615152b6aff047431f2e2e6267db6167275a5b45892fef773e5e2571a6` |
| SFC              |   487 | `34f1fe869137645c7b49db0a289a6586c33b02471aad6685d440ab61830ae15a` |
| tsconfig         |   310 | `463c6d61dd2328ea4ef074e7884cd2c803758b6fae8a1bfe882d00c9e898a2df` |
| lint config      |    93 | `b1ed60fd1f91bcc404fce797e7f3b0ab62457c5c95e8492bf572dac804d78e25` |
| Reported console |   420 | `143d2e0853386a75732ecb292e28b91aae99e8a0e13821ae8dc763da48277626` |

## Prepared qualification

Seven complete config laws cover absent/true/false, inherited true, explicit
false override, array-extends precedence, and nearest config ownership. Eleven
whole native/API/CLI cases preserve the original, false/absent controls,
inheritance/override/nearest cases, CRLF/Unicode, two allowance controls, and
known tuple/property indices. Whole outputs are authored before execution.
The original true flag allows nullable TouchList/Record object guards and
rejects only the nullable `items[0]` string. False/absent retain the two object
findings. Known tuple/property objects remain always truthy.

The source-built test requires all public diagnostic fields, full JSON/plain
stdout, empty complete stderr, status, and unchanged original/config bytes.
It captures the actual native session's whole config and generated source,
lossless process results before assertions, and CLI/native binary hashes.
Only the independently owned temporary root replaces the expected API filename.
The original lint config is unchanged; `CORSA_PATH` and the frozen Vue symlink
are explicit test bootstrap. Default help is qualified at the reported `none`
setting; no whole default/full-help equivalence is claimed.

The existing automatic native source CLI invocation also runs this test and
uploads its full capture. Ordinary PR Rust shards intentionally disable native
runtime; only required native/full-queue execution can qualify eleven API and
twenty-two CLI results. Current Rust compilation and runtime are pending.

## Remaining work and limits

TODO: execute fresh exact-source/native/protected suites before public delivery.
No local native build, source runtime, workflow campaign, public PR, merge,
release, or performance credit exists for this private preparation.

TODO: switching governing tsconfigs inside one package with a reused long-lived
Linter retains the existing project-root-based session cache rule. This change
does not rekey that cache or qualify parity for every compiler option outside
the existing whitelist. No additional source lane is started for those bounds.
