# Type-aware project batching corpus

This fixture preserves the original 121-file type-aware workload used to investigate
#10358: 40 Vue components, 80 TypeScript modules, and one shared composable. Three
templates reproduce every source byte; `authority.json` pins all 121 original hashes.
The benchmark creates ten unchanged cohorts, yielding 1,210 authored files.

`cohort.stdout` is the complete public 0.557.0 JSON output from the original cohort,
also reproduced by public 0.558.0. Only its relative file prefixes are extended for
the ten cohorts. Every diagnostic field, ordering, file entry, and raw JSON byte is
required to match. The expanded output is 1,704,822 bytes, contains 2,820 diagnostics,
and has SHA-256 `bf3a6089ca4026480ebcf5b188ae11f1e757c6067f59eea0fcb3ca121bcc50dc`.
The output deliberately has no trailing newline.

The frozen request vectors were observed with TypeScript 7.0.2. The former
single-document publication path has 1,211 `updateSnapshot` transports; the project
batch path has two: initial session creation and complete document publication.
Default-project requests likewise change from 1,211 to two, and snapshot releases
from 1,210 to one. The entire 31-phase vector is checked, including unchanged symbol,
type, property, signature, and return-type requests. Missing/unknown phases or
truncation fail qualification.

The optional `patina_project_batch` input in `benchmark.yml` reuses that workflow's
exact base/head `ci-opt` binaries. It installs the locked Vue 3.5.44 consumer outside
the timed region, runs a warmup per side and five alternating pairs, then collects
two separate profile pairs and one separate GNU time RSS observation per side. No
profile or RSS observation enters the latency estimator. Input membership, source
bytes, installed dependency bytes, compiler binaries, and checker binary are checked
again after the workload. GNU time's maximum resident set size is not a sampled sum
of concurrent process RSS; transport spans include queueing and wire/upstream work,
and do not report exclusive worker CPU time.

Dispatch against a branch containing both the batch implementation and this harness:

```sh
gh workflow run benchmark.yml --ref <branch> \
  -f base_sha=<full-before-sha> -f head_sha=<full-head-sha> \
  -f patina_project_batch=true
```

The before source must retain the original single-document request vector and both
sources must reproduce this complete output. Evidence is retained separately from
the ordinary benchmark report, including failed output observations. This fixture
does not itself establish an optimized latency improvement; that requires a successful
measurement artifact for the exact source pair.
