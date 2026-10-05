# Honor existing JSX quote preference (#7763)

Issue: [#7763](https://github.com/ubugeeei-prod/vize/issues/7763).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

After #7755 actually merged as signed `da8638a8`, a fresh wt worktree audits the
existing `jsxSingleQuote` setting in schema/generated config types, Glyph,
CLI/editor and WASM serde. The final shared Oxc projection omits jsx_quote_style;
pinned Oxc and Glyph false/default both select Double. Forward the existing bool
in that same constructor. Condense the existing Oxc-version/layout comment without
losing its meaning to keep the owner under 350. No new stage or API is introduced.

The omission is source-inspected; no executed old-baseline output is claimed.
Independent complete TSX/JSX and SFC references distinguish all four booleans,
omitted=false, ordinary JS strings and HTML attributes. Pinned Oxc StringLiteral
under JSXAttribute uses original source bytes and JSX quote style. Quote entities
and raw delimiters count equally; fewer escapes win and preference resolves ties.
Thus title="it's" retains double quotes while title="' &quot;" with true becomes
`title='&apos; "'`. Ordinary HTML uses the separate attribute serializer.
These controls prevent confusing an incorrect authored expectation with a bug.

The 207-byte TSX SFC has a 227-byte authored reference with singleQuote=false
and jsxSingleQuote=true (SHA5b84ea1a).
It becomes the eleventh configured CLI case; all ten prior cases/history captures,
300 API plans, complete native owner transition and original nine plans/25 calls remain
unchanged. Correct the current README's old Node-case label to its actual shared
raw-case name; no fixture identity/reference changes. Physical inputs remain
.vue.txt and consumer inventory remains unchanged. No native provider credit.

Public Rust TSX/JSX and Vue 2/2.7/3 SFC laws compare complete outputs, immutable
inputs/options and three passes, with structured changed=false after the first
SFC pass. Script APIs return code only. Source-map acceptance is not claimed.
Fresh exact-source Actions/all 104 unchanged ceilings, independent auto-squash,
full protected queue and actual signed merge are required. If a candidate fails,
remove it until repaired; stale-prefix receipts do not qualify regenerated heads.
Keep the verified reporter trailer in source and PR; inspect actual final credit
under the existing same-primary normalization policy. New publication is separate.

TODO: broad profiles, option combinations/source maps and Node quote-option
exposure remain open under #6098. Existing quoteProps/default/newline fixes stay
intact; this repair closes only the concrete shared JSX projection omission.
