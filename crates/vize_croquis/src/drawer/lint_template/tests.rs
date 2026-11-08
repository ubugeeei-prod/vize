use super::LintTemplate;
use crate::Drawer;
use vize_armature::Parser;
use vize_l0::Allocator;
use vize_l1_to_l2::lower_preserving_comments;

const CASES: &[&str] = &[
    "<div id=app :aria-controls=panel>{{ title }}<input ref=field></div>",
    "<Card :title @save.stop=submit><template #default=\"{ item }\">{{ item.name }}</template></Card>",
    "<ul><li v-for=\"(item, index) in items\" :key=item.id>{{ index }} {{ item.name }}</li></ul>",
    "<component :is=kind v-if=visible v-model.trim=value />",
    "<RouterLink to=\"/home\"/><RouterLink/><div v-pin:top.lazy=offset></div>",
    "<section v-bind=attrs><button @click=\"count++; save(count)\">{{ $attrs.id }}</button></section>",
    "<div v-pre :raw=\"ignored\">{{ literal }}</div><span v-else-if=other>{{ other }}</span>",
];

#[test]
fn l1_l2_lint_projection_matches_relief_semantics() {
    for source in CASES {
        let allocator = Allocator::default();
        let (legacy, legacy_errors) = Parser::new(&allocator, source).parse();
        assert!(
            legacy_errors.is_empty(),
            "legacy fixture must parse: {source}"
        );
        let (surface, errors) = vize_l1::markup::parse(&allocator, source);
        assert!(errors.is_empty(), "L1 fixture must parse: {source}");
        let lowered = lower_preserving_comments(&allocator, &surface, &errors);

        let mut expected = Drawer::for_lint();
        expected.draw_template(&legacy);
        let mut actual = Drawer::for_lint();
        actual.draw_lint_template(LintTemplate::new(&surface, &lowered));

        assert_eq!(
            actual.finish().semantic_snapshot(),
            expected.finish().semantic_snapshot(),
            "semantic mismatch for {source}"
        );
    }
}
