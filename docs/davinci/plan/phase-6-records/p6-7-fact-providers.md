# P6-7 — Custom JS fact providers

The provider hook supplies declared tables to native `lintWithPlugins` consumers.
Its Rust boundary lives in `vize_vitrine::napi::plugin_sdk::providers`; the authoring
entry point is `defineFactProvider` in `npm/plugin-sdk/index.js`.

## Manifest and closure

A provider declares `name`, `version`, `fingerprint`, `visit`, `demands`, `provides`
and `cacheInputs`. Ambient inputs require an explicit array, including an empty
array. Names beginning with `@vize/` are reserved for host cache dependencies.

Provider `tokens` owns names beginning with `tokens/`, for example `tokens/colors`.
The suffix is nonempty and the full group name contains no whitespace. A provider
cannot replace a native fact group or publish another provider's namespace.
Duplicate identities, duplicate group declarations, unknown dependencies and
cycles are refused before invoking a provider callback. Consumers resolve only
the declared transitive closure; providers outside that closure do not run.

## Batch and result

Each provider receives the normal serialized visit batch with exactly its
declared native and custom fact groups. A dependent rule receives exactly its
own demanded groups. Dependency tables remain private when the rule did not
declare them. The SDK freezes input batches and rejects asynchronous callbacks.

The result is a JSON object whose keys exactly match `provides`. Each value is a
table of `[string | u32, JSONValue]` pairs with unique keys. The maximum serialized
result is 4 MiB. Missing or additional groups, malformed rows, duplicate keys and
values outside the JSON format are rejected by the host boundary.

Providers execute synchronously on Node's calling thread with the application's
JS permissions. Authors declare configuration and ambient reads in `cacheInputs`.
One demanded execution invokes the callback twice with the same batch and compares
the parsed results, including table order. JSON object property order is canonical.
Different results refuse the lint request. Provider timing includes both calls.

## Reuse and evidence

An audited table is shared by consumers within the lint request. Each table's
consumer cache dependency includes the provider's source/configuration key and
its canonical result key. Version, code, ambient inputs, transitive dependencies
and changed result values invalidate cached diagnostics. Provider results have a
bounded process cache (32 entries, 16 MiB) and an optional atomic disk cache. Hits
validate the schema, complete key, declared tables and canonical result key before
reuse. Corrupt or stale entries become misses. A miss invokes and audits JS twice
before publishing a cached result. Provider costs distinguish `cached` from
`audited`; cached results have zero JS time.

`providers/tests.rs` covers closure, refusals, auditing, batch visibility and keys.
`tests/tooling/level-plugin-fact-providers.test.ts` exercises the actual SDK and
native host through a rule that reads a provided table and emits an anchored
diagnostic, multi-provider dependencies, cache invalidation and refusal cases.
`level-plugin-fact-provider-cache.test.ts` checks independent provider and rule
reuse across Node processes and recovery from a corrupted provider table.
The parent P6-7 record must include its complete Actions evidence before closing GA.
