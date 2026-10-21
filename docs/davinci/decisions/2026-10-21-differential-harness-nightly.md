# Nightly real-project differential coverage (#6891)

The real-project matrix exercises the registered ecosystem corpus with the
compiler, linter, formatter, typechecker, and LSP product surfaces. It now runs
nightly at `05:37 UTC`, matching the T2 cadence required by #6891. Manual
dispatch remains available for release and scoped diagnostic evidence.

The workflow contract pins the nightly cron alongside the existing 22-shard
selection and every-product execution checks. This prevents the full corpus
from silently returning to weekly coverage while keeping the workflow off the
pull-request path.
