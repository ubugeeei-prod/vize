# Retirement registry transport

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The official fresh-release operator [38142216801](https://github.com/ubugeeei-prod/vize/actions/runs/38142216801)
stopped before creating a source PR while checking the previously retired
`v0.444.0` at `1affd1cf175ad033858743d8ea923384dc076a38`. Its only network
diagnostic was `fetch failed`, so the failing registry and underlying cause are
unknown. No new release or public version resulted from this attempt.

Retirement now records the exact registry, package, URL, attempt count and
bounded escaped error/cause chain when a fetch rejects. Only recognized
temporary connection/reset/DNS timeout codes permit another anonymous read:
at most three attempts, with 250/500 ms backoff, under the same original
30-second per-target abort deadline. Unknown network errors, certificate
failures, permanent DNS failures and aborts remain terminal. Every received
HTTP response still enters the original exact-version status, URL, content,
size, schema and full Marketplace-inventory checks once; a publication,
redirect, rate-limit response or invalid absence proof remains fatal.

The inherited absence and timeout controls remain intact. Added controls cover
temporary recovery, exhausted retries, permanent/unknown/mixed causes, abort,
unchanged request/response identity, safe diagnostic lines and the actual
retirement caller. That caller fails before the integration repair and passes
after it. The focused complete public-acceptance and release-input selector
tests pass locally; fresh exact-head Actions and the protected merge precede
the next official start. A successful start still requires terminal publication
and fresh installed-consumer acceptance.
