# Art files are not component definitions

Paired decision: [#7823](https://github.com/ubugeeei-prod/vize/issues/7823#issuecomment-5987437053).

The two filename-only rules, `vue/component-definition-name-casing` and
`vue/multi-word-component-names`, exclude the physical case-sensitive
`*.art.vue` convention. A Musea Art file describes variants; its filename
does not define the demonstrated component. The existing registered native
casing callback uses the same exception after its original-owner check.

Keep the physical filename, original SFC parsing, every variant range and
its original coordinate offset. Do not strip `.art`, manufacture another
component filename, skip all variant rules or deduplicate real findings.
Ordinary `.vue` files retain their naming policies, including dotted
invalid names and differently cased suffixes such as `Badge.Art.vue`.
The component demonstrated by the Art file still has its own independent
ordinary `.vue` naming rules.

## Regression custody and acceptance

The original issue and follow-up sources are retained without formatting in
`crates/vize_patina/tests/fixtures/musea-component-names/`, with their source
URLs, lengths and SHA-256 hashes in `originals.json`. They are respectively
298 and 243 bytes. The API and actual CLI tests require complete empty
results under default and explicit filename-rule configuration.

Positive controls retain ordinary filename findings, both real variant
autofocus findings with UTF-8/CRLF physical spans, complete unrelated
diagnostics and opt-in Musea metadata findings. The native callback refuses
a separate original owner even when source bytes and spans match, before
the Art exception. Only the scanner-generated owned Patina census gains
the new dev-only L0 test reference; inventories and budgets are unchanged.

These source laws await exact-head Actions and protected/actual merge
proof. A separately prepared original-source mount probe uses genuine Art
processing and actual Vue; it is not yet a passing runtime receipt.
Script-binding and registration problems in [#7897](https://github.com/ubugeeei-prod/vize/issues/7897)
and [#7900](https://github.com/ubugeeei-prod/vize/issues/7900) are separate.
This filename correction changes no parser/compiler/variant context,
legacy-backed Davinci route, default migration, fix-history acceptance or
performance budget. Hold admission through the verified release thaw and
close the issue only after the correction actually merges with its gates.
