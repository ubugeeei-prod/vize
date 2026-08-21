//! The committed legacy battery and its counters (P2-9 series 7;
//! split from [`super::legacy`] under the source budget).

#![allow(dead_code)]

/// The committed legacy battery: every template names the legacy class
/// it pins. Run under the V2 dialect by [`compare_legacy`] and under
/// the default dialect by the witness's V3-meaning control.
pub const LEGACY_BATTERY: &[(&str, &str)] = &[
    ("filter-interp", r#"<div>{{ message | capitalize }}</div>"#),
    ("filter-chain-args", r#"<p>{{ a | f(b) | g }}</p>"#),
    ("filter-bind", r#"<span :id="raw | formatId"></span>"#),
    (
        "filter-quoted-pipe",
        r#"<span :title="'a|b' | quote"></span>"#,
    ),
    ("filter-logical-or", r#"<div>{{ a || b }}</div>"#),
    ("filter-bad-name", r#"<div>{{ x | (bad) }}</div>"#),
    (
        "filter-dedup",
        r#"<div>{{ a | f }}<i>{{ b | g | f }}</i></div>"#,
    ),
    ("filter-in-compound", r#"<em>pre {{ m | f }}</em>"#),
    ("filter-condition", r#"<div v-if="ok | truthy">c</div>"#),
    (
        "sync-basic",
        r#"<MyPane :title.sync="pane.title"></MyPane>"#,
    ),
    (
        "sync-multi-and-plain",
        r#"<Panel :a.sync="x" :b="y" @update:c="h"></Panel>"#,
    ),
    ("sync-dynamic-arg", r#"<Row :[k].sync="v"></Row>"#),
    ("sync-same-name", r#"<Widget :model-name.sync></Widget>"#),
    ("sync-native-element", r#"<input :value.sync="text">"#),
    (
        "native-keycodes",
        r#"<button @click.native.stop="go" @keyup.13="submit" @keydown.99.native="odd"></button>"#,
    ),
    (
        "scoped-slot-named",
        r#"<Card><template slot="header" slot-scope="props"><b>{{ props.title }}</b></template></Card>"#,
    ),
    (
        "scoped-slot-default-alias",
        r#"<List><template scope="row">item</template></List>"#,
    ),
    (
        "scoped-slot-conflict",
        r#"<Grid><template slot-scope="cell" #conflict="c">y</template></Grid>"#,
    ),
    (
        "plain-slot-attr",
        r#"<Tab><template slot="named">z</template></Tab>"#,
    ),
];

/// The legacy lane's own accounting, beside the shared [`Counters`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegacyCounters {
    /// Filter sites compared structurally against the shipped splitter.
    pub filter_sites: u64,
    /// Filter segments compared (text + name).
    pub filter_segments: u64,
    /// Templates whose armed-run registration equals the S2 assets.
    pub assets_matched: u64,
    /// Templates where the armed run registered strictly more (the
    /// recorded S2 narrowing; subset asserted, never averaged).
    pub assets_narrowed: u64,
    /// Probe: a position S2 deliberately does not split (condition,
    /// v-for source, v-on value, v-model value, custom-directive value)
    /// carries a splittable chain.
    pub filters_other_positions: u64,
    /// Probe: a compound unit's dynamic part carries a splittable
    /// chain (the merged-run narrowing).
    pub filters_in_compounds: u64,
    /// `.sync` expansions mirrored (S2 provenance `normalize.legacy.sync`).
    pub syncs: u64,
    /// Scoped-slot conversions mirrored (`normalize.legacy.slot-scope`).
    pub scoped_slots: u64,
    /// `.native` strips mirrored (`normalize.legacy.native`).
    pub natives: u64,
    /// Keycode renames mirrored (`normalize.legacy.keycode`).
    pub keycodes: u64,
}
