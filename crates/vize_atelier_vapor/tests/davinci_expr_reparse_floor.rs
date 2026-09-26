//! P1-7 legacy re-parse floor: the Vapor lane of the expr-reparse baseline.
//!
//! One fused compile per ladder fixture, counting the surviving legacy oxc
//! expression re-parses via the P0-3 probe — the same sweep that produced
//! `docs/davinci/plan/expr-reparse-baseline.md`, pinned exactly at the
//! post-P1-7 floor (the pre-P1-7 counts are the baseline doc's table). Any
//! change means a retained-consumption gate or a legacy site moved:
//! re-derive the floor deliberately and update the baseline doc's post-P1-7
//! section with it.
//!
//! The probe is process-global and monotone, so this file holds a single
//! `#[test]` (its own binary — no concurrent recorder), mirroring
//! `vize_armature/tests/davinci_expr_parses.rs`. Integration binaries
//! compile the library without `cfg(test)`, so the P1-7 differential
//! comparators are disarmed here and the deltas are the production floor.

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_core::{expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

/// fixture name → surviving legacy re-parses per fused Vapor compile.
const FLOOR: [(&str, u64); 6] = [
    ("small", 0),
    ("medium", 0),
    ("large", 8),
    ("stress-deep", 0),
    ("stress-wide", 0),
    ("stress-interp", 0),
];

#[test]
fn vapor_legacy_reparse_floor_holds() {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = expr_parse_probe::expr_parse_count();
        let _compiled = compile_vapor(&allocator, template, VaporCompilerOptions::default());
        let parses = expr_parse_probe::expr_parse_count() - before;
        let floor = FLOOR
            .iter()
            .find(|(name, _)| *name == fixture.name)
            .unwrap_or_else(|| panic!("ladder fixture {} has no pinned floor", fixture.name))
            .1;
        assert_eq!(
            parses, floor,
            "vapor {}: surviving legacy re-parses moved from the pinned P1-7 floor",
            fixture.name
        );
    }
    // Computed names retain their S2 expression AST. Keep this recorder in
    // the same single-test binary so process-global probes cannot race.
    for source in [
        r#"<slot :name="name"></slot>"#,
        r#"<slot :[name]="value" />"#,
        r#"<slot :name="selected" :[name]="value" />"#,
        r#"<slot data-id="fixed" :[names[index]]="value" :extra="extra" />"#,
        r#"<slot :['data-'+suffix]="value" :[name.toLowerCase()]="other" />"#,
        r#"<slot class="base" :class="classes" style="color:red" :style="styles" :[name]="value" />"#,
        r#"<Child v-slot="{ item }"><slot :[item.name]="item.value" /></Child>"#,
        r#"<slot data-id="fixed" disabled />"#,
        r#"<slot class="base" :class="classes" style="color:red" :style="styles" />"#,
        r#"<slot class :class="classes" />"#,
        r#"<div title="fixed" :[name]="value"></div>"#,
        r#"<div title="fixed" :[title]="value"></div>"#,
        r#"<div class="base" :class="classes" style="color:red" :style="styles" :[keys[index]]="value"></div>"#,
        r#"<div :[name.toLowerCase()]="value" v-bind="attrs" :['data-'+suffix]="extra"></div>"#,
        r#"<input value="fixed" :[name]="value">"#,
        r#"<ul><li v-for="item in items" :key="item.id" :data-id="item.id" title="fixed" :[item.name]="item.value">{{ item.label }}</li></ul>"#,
        r#"<button v-once :title="label" @click="save">{{ label }}</button>"#,
        r#"<main v-once><button @click.stop.prevent="record(label)">{{ label }}</button></main>"#,
        r#"<main v-once><button @focus.capture="save" @keydown.enter="record(label)">{{ label }}</button></main>"#,
        r#"<Transition :css="false"><p>{{ label }}</p></Transition>"#,
        r#"<Transition :css="false"><div v-if="show"><span>{{ label }}</span></div></Transition>"#,
        r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id"><span>{{ item.text }}</span></li></TransitionGroup>"#,
        r#"<Transition :css="false" @enter="enter" @leave="leave"><button v-if="show" @click="save">{{ label }}</button></Transition>"#,
        r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id">{{ item.text }}</li></TransitionGroup>"#,
        r#"<KeepAlive><MyComp /></KeepAlive>"#,
        r#"<KeepAlive include="First" max="2"><component :is="view" :label="label" @send="record" /></KeepAlive>"#,
        r#"<KeepAlive :include="names" :exclude="excluded" :max="limit"><component :is="views[selected]" v-model="value" /></KeepAlive>"#,
        r#"<Child :[propName]="value" @[eventName]="record" />"#,
        r#"<Child label="static" :[label]="value" @[eventName]="record" />"#,
        r#"<component :is="view" :[is]="value" @[eventName]="record" />"#,
        r#"<Child :[names[selected]]="value" @[events[selected]]="record(value)" />"#,
        r#"<Child :[name.toLowerCase()]="value" @['saved-'+suffix]="(v) => record(v)" />"#,
        r#"<slot :name="names[selected]" :value="count"><b>{{ fallback }}</b></slot>"#,
        r#"<slot :name="enabled ? first : second"></slot>"#,
        r#"<slot :name="'prefix-' + selected"></slot>"#,
        r#"<slot :name="name.toLowerCase()"></slot>"#,
        r#"<Child v-model:[name]="value" />"#,
        r#"<Child v-model:[names[index]].trim.number="form[field]" />"#,
        r#"<Child v-model:[name.toLowerCase()].custom="value" />"#,
        r#"<Child v-model:[enabled?first:second]="items[index]" />"#,
        r#"<Child v-model:['prefix-'+suffix].trim="form.title" />"#,
        r#"<component :is="view" title="static" v-model:[name].trim="value" />"#,
        r#"<Outer v-slot="{ item }"><Child v-model:[item.name].trim="form[item.key]" /></Outer>"#,
        r#"<component :is="view" v-model="value" />"#,
        r#"<component :is="view" v-model="items[index]" />"#,
        r#"<MyComp v-model:title.trim="form[field]" />"#,
        r#"<input v-model="items[index]">"#,
        r#"<textarea v-model="form[field]"></textarea>"#,
        r#"<component :is="view" v-model:title.trim="form.title" />"#,
        r#"<MyComponent v-slot:head="p">{{ p.x }}</MyComponent>"#,
        r#"<MyComponent v-slot:[name]="{ item }">{{ item }}</MyComponent>"#,
        r#"<MyComponent><template #[name]>x</template></MyComponent>"#,
        r#"<MyComponent><template #[names[selected]]="{ item }">{{ item }}</template><template #fixed>fixed</template></MyComponent>"#,
        r#"<MyComponent><template #['slot-'+selected]>x</template><template #[selected.toLowerCase()]>y</template></MyComponent>"#,
        r#"<button @[eventName]="save">go</button>"#,
        r#"<button v-on:[names[selected]].once.capture.passive="save">go</button>"#,
        r#"<button @[enabled?first:second].enter.stop="save">go</button>"#,
        r#"<button @[eventName.toLowerCase()].right="save">go</button>"#,
        r#"<Teleport to="body"><div>content</div></Teleport>"#,
        r#"<Teleport :to="target" :disabled="disabled" defer><button @click="save">{{ label }}</button></Teleport>"#,
        r#"<Teleport :to="targets[selected]" :defer="deferred"><span v-if="visible">{{ value }}</span><span v-else>fallback</span></Teleport>"#,
        r#"<Teleport to="body"><ul><li v-for="item in items" :key="item.id">{{ item.label }}</li></ul></Teleport>"#,
        r#"<select v-model="selected"><option value="a">A</option><option value="b">B</option></select>"#,
        r#"<select multiple v-model="selected"><option :value="first">{{ label }}</option><option :value="second">B</option></select>"#,
        r#"<select v-model.number="form.selected"><option value="1">One</option><option value="2">Two</option></select>"#,
        r#"<select v-model="values[key]"><option value="a">A</option><option value="b">B</option></select>"#,
        r#"<Child><template #one v-if="enabled">A</template><template #two v-else>B</template></Child>"#,
        r#"<Child><template #[names[selected]] v-if="enabled">A</template><template #two v-else-if="second">B</template><template #one v-else>C</template></Child>"#,
        r#"<Child><template v-for="item in items" #[item.name]><b>{{item.label}}</b></template></Child>"#,
        r#"<Child><template v-for="(item, key, index) in items" #[item.name]="{ value }"><button @click="record(item.label)">{{item.label}}:{{key}}:{{index}}:{{value}}</button></template></Child>"#,
        r#"<Child><template v-for="item in items" #[item.name]="item"><b>{{item.x}}</b></template></Child>"#,
    ] {
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let walks = WalkCounts::snapshot();
            let parses = expr_parse_probe::expr_parse_count();
            let compiled = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers,
                    ..Default::default()
                },
            );
            assert_eq!(compiled.error_messages.len(), 0, "{source}");
            assert_eq!(
                WalkCounts::snapshot().since(walks).total_walks(),
                0,
                "{source}"
            );
            assert_eq!(expr_parse_probe::expr_parse_count() - parses, 0, "{source}");
        }
    }
}
