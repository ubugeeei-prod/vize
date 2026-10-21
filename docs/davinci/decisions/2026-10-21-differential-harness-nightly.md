# Nightly real-project differential coverage (#6891)

The real-project matrix exercises the registered ecosystem corpus with the
compiler, linter, formatter, typechecker, and LSP product surfaces. It now runs
nightly at `05:37 UTC`, matching the T2 cadence required by #6891. Manual
dispatch remains available for release and scoped diagnostic evidence.

The workflow contract pins the nightly cron alongside the existing 22-shard
selection and every-product execution checks. This prevents the full corpus
from silently returning to weekly coverage while keeping the workflow off the
pull-request path.

The nightly selection now covers all 147 ecosystem gitlinks pinned at this
revision. The registry has 149 project rows because PrimeVue intentionally
provides three separately scoped projects from one gitlink. The only excluded
gitlink is `vue-benchmarks`, whose independent benchmark registry does not
grant ecosystem/App membership or product-oracle credit. The four former
App-only inputs (`frontend-phpcon-do-website`, `npmx.dev`, `nuxt-ui`, and
`vuefes-2025`) use their existing complete check/lint SFC scopes and now run
through compiler, linter, formatter, typechecker, syntax-highlighter, and LSP
nightly selection as well.

The registry contract derives this membership from the committed gitlinks, so
a future ecosystem submodule cannot silently remain outside T2. This closes
the known corpus-selection omission only. Product-specific legacy/native
comparators and complete paired T2 differential execution remain open under
#6891.
