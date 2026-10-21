//! A separate, nonserialized result of the existing script parse demand.

use super::Drawer;
use crate::script_parser::ModuleExportCapture;

impl Drawer {
    /// Observe module exports during the same original OXC parse.
    #[doc(hidden)]
    pub fn with_empty_module_exports(mut self) -> Self {
        let mut capture = ModuleExportCapture::default();
        if self.script_drawn {
            capture.refuse();
        }
        self.module_export_capture = Some(capture);
        self
    }

    /// Return ordinary Croquis unchanged and a bounded empty-export proof.
    #[doc(hidden)]
    pub fn finish_with_empty_module_exports(self) -> (crate::Croquis, bool) {
        let empty = self
            .module_export_capture
            .as_ref()
            .is_some_and(ModuleExportCapture::known_empty);
        (self.finish(), empty)
    }

    /// Return ordinary Croquis unchanged and every authored named export, or
    /// `None` when the module's export set is not provable from its own source.
    #[doc(hidden)]
    pub fn finish_with_module_exports(
        self,
    ) -> (
        crate::Croquis,
        Option<Vec<crate::script_parser::AuthoredModuleExport>>,
    ) {
        let exports = self
            .module_export_capture
            .as_ref()
            .and_then(ModuleExportCapture::known_exports);
        (self.finish(), exports)
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::disallowed_macros,
        reason = "fixtures assert complete debug and source contracts"
    )]

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::{Croquis, Drawer, DrawerOptions};
    use vize_carton::CompactString;

    fn assert_contract(ordinary: &Croquis, captured: &Croquis) {
        assert_eq!(format!("{ordinary:#?}"), format!("{captured:#?}"));
        assert_eq!(ordinary.to_vir(), captured.to_vir());
        assert_eq!(
            serde_json::to_value(ordinary.semantic_snapshot()).unwrap(),
            serde_json::to_value(captured.semantic_snapshot()).unwrap()
        );
    }

    fn draw(drawer: &mut Drawer, source: &str, setup: bool) {
        if setup {
            drawer.draw_script_setup(source);
        } else {
            drawer.draw_script_plain(source);
        }
    }

    fn assert_capture(source: &str, setup: bool, expected_empty: bool) -> Croquis {
        let mut ordinary = Drawer::with_options(DrawerOptions::full());
        let mut captured = Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
        draw(&mut ordinary, source, setup);
        draw(&mut captured, source, setup);
        let ordinary = ordinary.finish();
        let (captured, empty) = captured.finish_with_empty_module_exports();
        assert_eq!(empty, expected_empty, "setup={setup}, source={source}");
        assert_contract(&ordinary, &captured);
        captured
    }

    #[test]
    fn actual_plain_and_setup_parses_admit_known_empty_private_modules() {
        for source in [
            "",
            "// export const decoy = 1;\nconst text = 'export * from fake';",
            "interface Private { label: string }; const local = '雪';",
            "import type { Props } from './types'; import Card from './Card.vue'; const local = 1;",
        ] {
            for setup in [false, true] {
                assert_capture(source, setup, true);
            }
        }
    }

    #[test]
    fn only_normal_script_may_admit_default_only_or_empty_named_exports() {
        for source in [
            "export {};",
            "export default { name: 'OnlyDefault' };",
            "export default function Component() { return 1; }",
            "export default class Component {}",
            "const component = {}; export { component as default };",
            "const component = {}; export { component as 'default' };",
        ] {
            assert_capture(source, false, true);
            assert_capture(source, true, false);
        }
    }

    #[test]
    fn actual_type_value_indirect_and_star_exports_all_refuse_empty_authority() {
        for source in [
            "export const named = 1;",
            "const local = 1; export { local };",
            "export function named() {}",
            "export class Named {}",
            "export type Public = string;",
            "export interface Public { value: string }",
            "type Public = string; export type { Public };",
            "interface Public { value: string }; export { type Public };",
            "export { default } from './Other.vue';",
            "import Imported from './Other.vue'; export { Imported as default };",
            "export * from './other';",
            "export type * from './types';",
            "export * as namespace from './other';",
            "export type * as Types from './types';",
            "export const { first, nested: { second } } = { first: 1, nested: { second: 2 } };",
        ] {
            for setup in [false, true] {
                assert_capture(source, setup, false);
            }
        }
    }

    #[test]
    fn private_setup_types_keep_real_macro_and_binding_contracts() {
        let source = "interface Private { label: string }; const props = defineProps<Private>(); const emit = defineEmits<{ save: [value: string] }>();";
        let croquis = assert_capture(source, true, true);
        assert_eq!(croquis.get_props().collect::<Vec<_>>(), [("label", true)]);
        assert_eq!(croquis.get_emits().collect::<Vec<_>>(), ["save"]);
        let mut bindings = croquis
            .bindings
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), *kind))
            .collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(name, _)| *name);
        assert_eq!(
            bindings,
            [
                ("emit", crate::BindingType::SetupConst),
                ("label", crate::BindingType::Props),
                ("props", crate::BindingType::SetupReactiveConst),
            ]
        );
    }

    #[test]
    fn unsupported_module_statements_and_malformed_authored_source_refuse() {
        for source in [
            "declare namespace Ambient { const hidden: number; }",
            "namespace Local { export const value = 1; }",
            "import Alias = require('./dependency');",
            "export = {};",
            "export as namespace GlobalThing;",
            "const value = ;",
            "export const broken = ",
            "const unclosed = '",
        ] {
            for setup in [false, true] {
                assert_capture(source, setup, false);
            }
        }
    }

    #[test]
    fn recovered_editing_view_never_authenticates_the_original_module() {
        let source = "const slots = {}; const 雪 = 1; slots. /* editing */;";
        let allocator = Allocator::default();
        let original = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(original.panicked);
        let recovered =
            crate::script_parser::parse_program_for_analysis(&allocator, source, SourceType::ts());
        assert!(!recovered.panicked);
        assert!(!recovered.diagnostics.is_empty());
        assert_eq!(recovered.program.source_text, source);
        for setup in [false, true] {
            assert_capture(source, setup, false);
        }
    }

    #[test]
    fn no_observation_disabled_demand_and_late_capture_never_claim_empty() {
        let ordinary = Drawer::new().finish();
        let (captured, empty) = Drawer::new()
            .with_empty_module_exports()
            .finish_with_empty_module_exports();
        assert!(!empty);
        assert_contract(&ordinary, &captured);
        for setup in [false, true] {
            let mut options = DrawerOptions::full();
            options.analyze_script = false;
            let mut ordinary = Drawer::with_options(options);
            let mut captured = Drawer::with_options(options).with_empty_module_exports();
            draw(&mut ordinary, "const local = 1;", setup);
            draw(&mut captured, "const local = 1;", setup);
            let (captured, empty) = captured.finish_with_empty_module_exports();
            assert!(!empty);
            assert_contract(&ordinary.finish(), &captured);

            let mut ordinary = Drawer::with_options(DrawerOptions::full());
            let mut captured = Drawer::with_options(DrawerOptions::full());
            draw(&mut ordinary, "const first = 1;", setup);
            draw(&mut captured, "const first = 1;", setup);
            let mut captured = captured.with_empty_module_exports();
            draw(&mut ordinary, "const second = 2;", setup);
            draw(&mut captured, "const second = 2;", setup);
            let (captured, empty) = captured.finish_with_empty_module_exports();
            assert!(!empty);
            assert_contract(&ordinary.finish(), &captured);
        }
    }

    #[test]
    fn missing_capture_and_inherited_drawn_croquis_remain_unqualified() {
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_script_setup("const local = 1;");
        let (croquis, empty) = drawer.finish_with_empty_module_exports();
        assert!(!empty);
        let original_debug = format!("{croquis:#?}");
        let original_vir = croquis.to_vir();
        let inherited =
            Drawer::with_croquis(DrawerOptions::full(), croquis, true).with_empty_module_exports();
        let (croquis, empty) = inherited.finish_with_empty_module_exports();
        assert!(!empty);
        assert_eq!(format!("{croquis:#?}"), original_debug);
        assert_eq!(croquis.to_vir(), original_vir);
    }

    #[test]
    fn refusing_one_original_block_is_sticky_in_both_draw_orders() {
        for rejected in [
            "export interface Public { value: string }",
            "export const publicValue = 1;",
            "const broken = ;",
        ] {
            for rejected_first in [false, true] {
                let private = "interface Private { label: string }; const local = 1;";
                let (plain, setup) = if rejected_first {
                    (rejected, private)
                } else {
                    (private, rejected)
                };
                let mut ordinary = Drawer::with_options(DrawerOptions::full());
                let mut captured =
                    Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
                ordinary.draw_script_plain(plain).draw_script_setup(setup);
                captured.draw_script_plain(plain).draw_script_setup(setup);
                ordinary.draw_script_setup("const last = 1;");
                captured.draw_script_setup("const last = 1;");
                let (captured, empty) = captured.finish_with_empty_module_exports();
                assert!(!empty);
                assert_contract(&ordinary.finish(), &captured);
            }
        }
    }

    #[test]
    fn reenabling_capture_cannot_replace_an_already_refused_original_source() {
        let mut ordinary = Drawer::with_options(DrawerOptions::full());
        let mut captured = Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
        ordinary.draw_script_plain("export const named = 1;");
        captured.draw_script_plain("export const named = 1;");
        let mut captured = captured.with_empty_module_exports();
        ordinary.draw_script_setup("const privateValue = 2;");
        captured.draw_script_setup("const privateValue = 2;");
        let (captured, empty) = captured.finish_with_empty_module_exports();
        assert!(!empty);
        assert_contract(&ordinary.finish(), &captured);
    }

    #[test]
    fn setup_exports_stickily_refuse_after_a_normal_default_and_before_private_setup() {
        for setup in [
            "export {};",
            "export default {};",
            "const component = {}; export { component as default };",
        ] {
            let mut ordinary = Drawer::with_options(DrawerOptions::full());
            let mut captured =
                Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
            ordinary
                .draw_script_plain("export default {};")
                .draw_script_setup(setup);
            captured
                .draw_script_plain("export default {};")
                .draw_script_setup(setup);
            ordinary.draw_script_setup("interface Private { label: string }; const local = 1;");
            captured.draw_script_setup("interface Private { label: string }; const local = 1;");
            let (captured, empty) = captured.finish_with_empty_module_exports();
            assert!(!empty);
            assert_contract(&ordinary.finish(), &captured);
        }
    }

    #[test]
    fn parse_free_caller_program_refuses_even_after_an_authentic_empty_parse() {
        let source = "interface Private { label: string }; const local = 1;";
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(!parsed.panicked && parsed.diagnostics.is_empty());
        for authentic_first in [false, true] {
            let mut ordinary = Drawer::with_options(DrawerOptions::full());
            let mut captured =
                Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
            if authentic_first {
                ordinary.draw_script_plain("export default {};");
                captured.draw_script_plain("export default {};");
            }
            ordinary.draw_script_setup_program(&parsed.program, source, None);
            captured.draw_script_setup_program(&parsed.program, source, None);
            ordinary.draw_script_setup("const later = 2;");
            captured.draw_script_setup("const later = 2;");
            let (captured, empty) = captured.finish_with_empty_module_exports();
            assert!(!empty);
            assert_contract(&ordinary.finish(), &captured);
        }
    }

    fn known_exports(source: &str, setup: bool) -> Option<Vec<(CompactString, bool)>> {
        let mut drawer = Drawer::with_options(DrawerOptions::full()).with_empty_module_exports();
        draw(&mut drawer, source, setup);
        let (_, exports) = drawer.finish_with_module_exports();
        exports.map(|exports| {
            exports
                .into_iter()
                .map(|export| (export.name, export.type_only))
                .collect()
        })
    }

    #[test]
    fn normal_script_named_exports_are_listed_with_their_type_or_value_meaning() {
        let source = "import type { Shared } from './shared'; \
        export interface AccordionProps { value: string } \
        export type AccordionEmits = { change: [] }; \
        export const accordionTheme = {}; \
        function helper() {} \
        class Box {} \
        export { helper, Box as Boxed, type Shared as SharedAlias }; \
        export { Shared }; \
        export default {};";
        assert_eq!(
            known_exports(source, false),
            Some(vec![
                ("AccordionProps".into(), true),
                ("AccordionEmits".into(), true),
                ("accordionTheme".into(), false),
                ("helper".into(), false),
                ("Boxed".into(), false),
                ("SharedAlias".into(), true),
                ("Shared".into(), true),
            ])
        );
        assert_eq!(known_exports("export default {};", false), Some(vec![]));
    }

    #[test]
    fn unprovable_named_exports_leave_the_export_set_unknown() {
        for source in [
            "import { Value } from './other'; export { Value };",
            "export * from './other';",
            "export { named } from './other';",
            "export const { first } = { first: 1 };",
            "const local = 1; export { local as 'quoted name' };",
            "export namespace Space { export const value = 1; }",
        ] {
            assert_eq!(known_exports(source, false), None, "source={source}");
        }
        assert_eq!(known_exports("export interface Props {}", true), None);
    }
}
