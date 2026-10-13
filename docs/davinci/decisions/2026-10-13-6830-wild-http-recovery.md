# Same-byte Wild recovery after an exhausted HTTP 5xx download

Related: [CI #6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original NoMut source Check 38261651668 refused before Playground and
canonical SSR-Pug ran. Jobs 114853696675 and 114853697347 each received HTTP
504 on the original Wild download, exhausted the initial request plus five
configured retries, and exited 22. Each setup step took about 42 seconds.
The dependent DOM gate refused the failed canonical prerequisite; this does
not identify a DOM product failure or the provider outage's cause.

The selected source was a68c4892f535ae397c75cc7a6060fec0609553cd. The original
upstream action remains pinned to 0bbbfa5df4380cab8e63cb8505a1ce65e1d10203;
its complete 1,706-byte source SHA256 is
80d1b927f19f7df0e00d99b44361104c13f2221d630bbd080f7dc6eb066fe61a.
Its original download had no checksum or additional version probe.

## Finite transport change

Only PR Playground and canonical SSR-Pug call the local adapter. Its owned
branch requires Linux, X64, version 0.9.0 and the original five retries.
Every other host, version or retry input delegates to the same pinned
upstream action. The selected branch uses literal version/retry values to
avoid interpolating action inputs into shell source.

The primary URI and original curl flags, retry count, exponential backoff
and stderr remain. An added write-out observes only the final HTTP status;
a sentinel preserves trailing newlines so malformed statuses cannot be
mistaken for a permitted value. Only curl exit 22 with exactly HTTP
500, 502, 503 or 504 permits one anonymous GitHub API asset download:
release 328403095, asset 428152085. The API request has fixed octet-stream
and API-version headers and no retry or additional alternative. API failure
preserves its nonzero status even when inherited errexit is disabled.
Authorization, absence, rate, TLS/network and malformed-status failures
remain refusals.

Both primary success and recovered success must match the entire original
2,445,608-byte Linux x64 archive's fixed SHA256:
deb6ee0e5caec798053ec4aafaba042e20a8edf91f08cb4d36268571cc628d3b.
The official immutable release metadata and complete primary/API bodies
independently matched that digest. This is additional custody before the
original tar extraction. No downloaded program is executed by the helper.
The original architecture mapping, mkdir failure, tar strip-components,
archive removal, existing-target grep and GNU/musl Cargo linker settings
remain. Other Wild callers, builds, tests, providers and deadlines remain.

The API asset route may redirect to the same release-assets provider.
Neither a shared provider root cause nor universal route availability is
established. ARM archives have no qualification from this x64 comparison.

## Controls and qualification

Thirteen bounded Node laws execute real Bash, GNU SHA256 and tar with an
inert authored archive and fake curl processes. The extracted Wild file is
never run. A private copied helper substitutes exactly one fixed digest
for the authored archive and proves the whole source inverse; production
has no digest override. Complete primary stdout, stderr, filesystem modes,
file bodies and Cargo configuration match the original expanded recipe.
All four admitted 5xx cases retain original failure before demonstrating
candidate extraction. Existing install-directory failure and preexisting
GNU/musl configurations retain original behavior. Wrong/truncated bodies,
changed member bytes/modes, API nonzero status with otherwise valid bytes,
malformed HTTP, foreign contexts and TLS failures refuse before tar.

The complete upstream action is retained as an authenticated fixture.
Reverse expansion restores its whole parsed body, including input defaults.
The two workflow changes preserve every other incoming step and condition;
the original source workflow remains 350 lines. The companion and tiny
canonical suffix preserve all other 349 incoming rows and the existing
owned paragraph prefix.

These source/process controls establish no hosted recovery, native
acceptance, causal timeout explanation, measured speed improvement or
installed release. Ordinary exact-source Actions and all protected suites
remain required. Original failures are retained; the waiting product lanes
consume this repair only after an actual qualified merge, with genuine
fresh-source composition rather than an unchanged failed-job retry.
