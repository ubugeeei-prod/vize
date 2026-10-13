# Japanese labels for the remaining guide navigation

Issue: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).

The actual Japanese contribution page at published source
`130331f713bdfa1db41abce506e0f2ce2ee5486f` has the authored heading
「貢献する」 while its sidebar still says “Contributing”. The navigation label
resolver has no Japanese entry for this route and falls back to the generated
English label. Five other existing guide routes have the same missing override.

Add six explicit labels in the existing Japanese locale map: contribution,
compiler inspector, experimentals, experimentals reference, Vue RFC details
and global preview controls. Use their authored Japanese page titles; the RFC
label follows the complete authored review in #8680. The label does not change
that feature's support or publication status. Preserve all original map entries,
route destinations, groups, all original locale entries, blog-title fallback,
language selection, navigation behavior and the disabled API-docs generator.

The first exact-source Actions run rejects Japanese-only additions because
the existing five-locale navigation contract requires identical complete key
vectors in the same order. Retain that guard and add the same six keys in
English, Chinese, French and Portuguese. Contribution and Inspector use each
existing authored page title. Where those locales have no experimental or
global-controls page, keep the original English fallback label explicitly;
adding a label does not add or advertise an unavailable translated route.
All original entries remain exact. The actual existing key/order test now
passes; fresh exact-head Actions remain required. The earlier unrelated real
Docker sandbox timeouts are retained as failed evidence, with no CI or test
timeout changes and no completion credit from a repeat.

This change requires its own exact-source Actions, protected actual merge,
full Docs and public sidebar reading proof. The existing public pages qualify
the observed missing labels, not the candidate's delivery. The complete
Japanese site review remains open; this slice does not review other prose or
claim any generated API-label instance that the site does not render.

The [existing contribution review](./2026-10-11-japanese-contribution-credits-review.md)
and [testing flow](./2026-10-11-testing-feedback-reader-flow.md) now retain their
scoped actual public reader acceptance against the exact qualified source130.
The six finite native cases establish the previously published body delivery;
they retain the old English sidebar as the genuine navigation counterexample.
