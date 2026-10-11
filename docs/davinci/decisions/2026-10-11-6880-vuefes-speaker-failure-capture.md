# VueFes speaker failure evidence

This decision belongs to [compiler fix history #6880](https://github.com/ubugeeei-prod/vize/issues/6880)
and the unfinished original VueFes qualifier on [PR #8609](https://github.com/ubugeeei-prod/vize/pull/8609).

[Actions run 38138497563](https://github.com/ubugeeei-prod/vize/actions/runs/38138497563)
tested head `79225770d823c0da9722d9fe80dcc542a47a7d5b`. Its unchanged
38-case visual matrix passed five routes, failed `dev-speaker`, and left
32 routes unrun. All three speaker attempts reported a difference ratio
of `0.030386641678062123` against the original `0.002` budget. The saved
images show the first eight Session speaker headshots replaced by the
eight Lightning Talk headshots, while the Session names and the later
speaker cards remain correct.

The existing passive observer only attached to `dev-home`. Consequently,
the failure artifacts contain the speaker screenshots but no speaker
document, DOM, or decoded script-response packets. They cannot establish
whether the first wrong image URLs originated in SSR, module evaluation,
or hydration. A local control using the complete pinned speaker sources,
Vue `3.6.0-alpha.2`, and simplified Nuxt components produced all 29 correct
SSR image URLs; it does not qualify the actual Nuxt integration.

Attach the same passive observer to `dev-speaker` before the existing
navigation, and publish its packets under a separate `vuefes-speaker`
artifact directory only after the original matcher fails. Keep the
existing `dev-home` capture directory. The observer retains its native
build authentication, five-second post-failure limit, original-error
rethrow, and refusal behavior. It does not alter the original route
matrix, setup, screenshot calls, budgets, retries, or test timeout.

The policy regression evaluates the actual observer selection against
all 38 original route and mode combinations. It requires capture of both
observed dev failures and distinct artifact directories, while the
existing corpus witness verifies the complete protected original bodies.

This change provides diagnosis, not a compiler fix or visual acceptance.
The remaining work is to run the composed exact head on Actions, compare
the actual speaker HTML and served modules with the hydrated DOM, add a
durable product regression for the identified cause, and pass the complete
unchanged 38-case qualifier before queue admission. Keep the known-failing
top layer out of the protected merge queue until then.
