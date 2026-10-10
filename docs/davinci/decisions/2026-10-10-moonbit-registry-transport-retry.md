# MoonBit registry transport retry (2026-10-10)

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed failure

[Check run 38058884169, job 114233357266](https://github.com/ubugeeei-prod/vize/actions/runs/38058884169/job/114233357266)
failed during the cold-cache setup action, before application tests. The pinned
MoonBit `0.10.7+bc794d341` installed successfully, but its initial `moon update`
failed to clone `https://mooncakes.io/git/index/`: Git exited 128 after HTTP 504,
and Moon exited 255. The same finite concurrency sample contains 31 cold-cache
setup jobs with registry 504 failures. Independent whole-log verification joins
all 31 cold misses to registry clone failure, Git128 and Moon255: three include
the exact URL fatal, and 28 contain RPC HTTP504/curl22 plus expected-packfile
failure without printing the URL. Both complete observed profiles are admitted.
This proves admission of the observed failure class, not successful recovery of
all historical jobs. A later
unchanged-source native setup
succeeded on its second Actions attempt; that is one recovery witness, rather
than qualification of every failed run.

## Decision

Retry only the existing cold-install registry update, with at most three total
attempts and fixed delays of 1 second and 2 seconds. Admission requires the
Moon update failure, a registry clone/fetch error, Git exit 128, Git stderr, and
either an exact fatal error from the existing mooncakes URL with HTTP 502, 503,
or 504, or the complete RPC HTTP502/503/504 curl22 plus expected-packfile profile
with outer Moon exit255. The RPC HTTP and returned-error status must agree, and its clone destination
must be the unchanged MOON_HOME registry/index path (native or slash-normalized
Git spelling). Its stderr does not independently prove the remote hostname. The
private helper has one caller: the already version-validated pinned executable
performing the existing cold-install registry update. It is not reused by other
bootstrap stages. A printed different Git URL is terminal.
Authentication, authorization, certificate, validation, installer hash, compiler
version, and compilation errors are terminal, including when accompanied by a
transient-looking line. Other HTTP errors, URLs, Git statuses, spawn failures,
and signaled children are terminal.

Each attempt keeps the same executable, argv, environment, registry location,
and package resolution. Child stdout remains inherited; stderr is copied as raw
bytes to the original stderr stream while it is captured for admission. The
parent awaits all stderr write callbacks before deciding or exiting, including
when stderr is a POSIX pipe. Every
attempt therefore remains visible in the Actions log. On exhaustion, the last
child exit status is retained. A spawn error or signal retains the original
installer fallback exit status 1.

The compiler pin, downloaded installer SHA-256 checks, version validation,
installation branches, cache paths and key schema, warm-cache behavior, shims,
Darwin header patch, native smoke test, and cache publication barrier are
unchanged. The existing installer-content hash naturally selects a new cache
key. There is no alternate registry, index deletion, skipped update, or package
fallback. There is no new child deadline: existing Actions job timeouts remain
the outer bound, and a hanging transport attempt remains subject to that bound.

## Validation and limits

Before changing the installer, authored controlled-child tests executed the
original one-shot body: 18 controls passed and six recovery/bounded-call laws
failed. The control uses real child processes with a fake Moon executable,
without installing MoonBit or accessing the network. The changed installer
passes all 24 controls, covering clone/fetch recovery, three-attempt exhaustion,
last status, whole raw stdout/stderr including non-UTF-8 bytes, exact argv and
environment with spaces and Unicode paths, authentication and validation
rejection, unrelated transport rejection, first-attempt success, signals,
missing executables, and the existing cold-install call and smoke-test order.

Twenty-one supplemental actual-child controls preserve those original 24 controls
and strengthen whole-output and integration evidence. Before the drain change,
the slow-reader control retained only 196,608 of 2,097,430 expected stderr bytes;
after awaiting tee callbacks it retains the complete bytes and terminal status.
The supplement checks exact ordered streams, distinct inner child PIDs, inherited
stdin consumed once, backoff timing, and a live stderr rendezvous in which the
inner Moon child waits for a reader acknowledgement before it may exit. Before admitting RPC failures, three
new recovery controls failed and 16 supplemental controls passed. The RPC
controls require the complete owned profile and reject other HTTP/curl codes,
mismatched HTTP values, other Moon exits, missing packfile errors, generic
failures, other Git operations/destinations and mixed authentication/compiler failures.
Controlled Unix cold/warm installer runs exercise the actual hash verifier,
version check, update, smoke stdin and export barrier with fake curl/bash/moon
children. Only the expected Unix installer digest in a temporary source copy is
injected for a local payload; the production digest remains unchanged. Tampered
payload and wrong-version controls fail before update, smoke and exports. These
Unix integration controls skip Windows; the retry child controls are portable.
This validates controlled stage behavior, rather than authenticating a real
download or compiler installation.

These controls qualify retry behavior, not upstream availability. Ordinary
exact-head Actions and the protected queue must still pass. The separate
release PTY timing diagnosis and feature concurrency work retain their owners
and existing acceptance criteria.
