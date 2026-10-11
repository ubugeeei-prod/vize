# VueFes development comment hydration

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

The observer ran on exact head `61970dd2ffd7cf844fbe88200ebfd022a8314020`
in [Actions run 38143695149](https://github.com/ubugeeei-prod/vize/actions/runs/38143695149).
The original speaker screenshot failed again. Its retry-zero final packet
is complete, has no missing facets, and identifies that head and the pinned
VueFes source `8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6`.

The candidate SSR document contains all 29 correct headshots and omits the
leading `eslint-disable-next-line` comment. The actual served speaker
client module emits that comment before the heading. Hydration therefore
aligns different child lists: it removes the first Session section, treats
Lightning Talk cards as Session cards, repairs their text, and retains the
old image attributes. The captured candidate DOM shows exactly the first
eight wrong photos from the screenshots. The reference document, module,
and DOM keep the correct pairs. A real local Nuxt `4.0.3` hydration with the
complete original speaker/data sources and Vue `3.6.0-alpha.2` reproduces
the same mismatch; that local experiment used the public `0.443.0` native
addon and does not substitute for exact-source qualification.

The minimized regression belongs to [#8652](https://github.com/ubugeeei-prod/vize/issues/8652).
Omit lint/Vize annotations at the selected native DOM emitter's existing
child-emission boundary. Keep annotation ops during lowering and transform:
the lint facade needs their custody, and ordinary development comments
affect whitespace, text flags and static analysis. Consume the omitted
ops' original page IDs so later bindings, text facts and slot facts remain
aligned. No additional stage or serialization is introduced. The existing
SSR compiler and its byte output remain unchanged.

Dropping annotations during lowering was rejected: it added a TEXT patch
flag to `a <!--eslint-disable rule--> b {{ value }}`, whereas the existing
transform observes the annotation and omits that flag. Complete selected
DOM/compatibility output comparisons pin this neighboring behavior.
Compatibility component-slot and template-list codegen also leave an array hole for a
stripped annotation. Preserve that legacy implementation; the corrected
native slot output is pinned to the complete plain-slot counterpart and
to real SSR hydration, where an absent annotation must not create a child.

The source-owned differential corpus contains the minimized leading
annotation template and a complete SFC with two groups of images and names.
The Rust regressions compare complete DOM output, preserve ordinary and
similar-looking comments, and execute complete native development DOM/SSR
modules through real Vue hydration. They check the original SSR pairs,
hydrated pairs, retained ordinary comments and hydration warnings across
all eight eslint/oxlint pragmas and ordinary-comment controls.

Remaining acceptance: qualify the composed exact source head with normal
Actions and the entire unchanged 38-case VueFes matrix before queue
admission, then carry the actual merged fix through an official release
and installed consumer acceptance. Keep the top layer draft and outside
the protected queue until those pre-merge checks pass. Prior failed runs
remain diagnostic evidence, not visual acceptance.
