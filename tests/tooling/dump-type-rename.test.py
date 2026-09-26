"""Pure namespace and wire-preservation checks for the bounded rename script."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('dump_rename', ROOT / 'tools/support/levels/rename-dump-types.py')
RENAME = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RENAME)
TARGETS = RENAME.names.targets({
    'crates/vize_l2/src/folio/owned.rs': 'pub enum FolioOp {}\npub struct FolioAttribute {}',
    'crates/vize_l2/src/folio/owned/expr.rs': 'pub enum FolioExpr {}',
    'crates/vize_l3/src/folio.rs': 'pub struct L3Folio {}\npub struct FolioOp {}',
})


class DumpRenameTests(unittest.TestCase):
    def test_shadow_and_live_ops_keep_distinct_bindings(self):
        result = RENAME.rewrite('crates/vize_l2/src/folio/owned.rs',
            'use crate::op::{Attribute, DynamicName, Namespace, Op, Region};\n'
            'pub enum FolioOp { Item(FolioAttribute) }\n'
            'pub struct FolioAttribute {}\nfn mirror(op: &Op) -> FolioOp { todo!() }', TARGETS)
        self.assertIn('Op as IrOp', result)
        self.assertIn('Attribute as IrAttribute', result)
        self.assertIn('fn mirror(op: &IrOp) -> Op', result)

    def test_cross_level_pages_keep_unambiguous_aliases(self):
        result = RENAME.rewrite('crates/consumer/src/lib.rs',
            'use vize_l2::folio::L2Folio;\nuse vize_l3::folio::L3Folio;\n'
            'fn pages(a: L2Folio, b: L3Folio) {}', TARGETS)
        self.assertIn('vize_l2::dump::Page as L2Page', result)
        self.assertIn('vize_l3::dump::Page as L3Page', result)
        self.assertIn('fn pages(a: L2Page, b: L3Page)', result)

    def test_collision_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'colliding alias Page'):
            RENAME.use_statement('crates/consumer/src/lib.rs',
                'use root::{vize_l2::folio::L2Folio as Page, vize_l3::folio::L3Folio as Page};', TARGETS)
        with self.assertRaisesRegex(ValueError, 'colliding declaration'):
            RENAME.names.targets({'crates/vize_l3/src/folio.rs': 'pub struct Page {} pub struct L3Folio {}'})

    def test_runtime_and_raw_fixture_literals_stay_exact(self):
        source = 'use vize_davinci::folio::FolioError;\n' \
            'const FIXTURE: &str = r#"[s3-folio]\nFolioError::new impeto.op"#;\n' \
            'fn message() { panic!("folio parse error"); }'
        result = RENAME.rewrite('crates/consumer/src/lib.rs', source, TARGETS)
        self.assertIn('r#"[s3-folio]\nFolioError::new impeto.op"#', result)
        self.assertIn('panic!("folio parse error")', result)

    def test_default_plan_header_is_pinned_before_nominal_rename(self):
        result = RENAME.rewrite('crates/vize_davinci/src/folio/plan.rs',
            '#[derive(Folio)]\npub struct FusionPlanFolio {}', TARGETS)
        self.assertIn('#[derive(Dump)]\n#[dump(name = "fusion-plan-folio")]\npub struct Page', result)

    def test_macro_uses_canonical_trait_and_mode_paths(self):
        result = RENAME.rewrite('crates/vize_davinci/src/folio.rs',
            'macro_rules! assert_folio_snapshot { () => { $crate::folio::Folio::print_to_string('
            '&value, $crate::folio::FolioMode::Full) }; }', TARGETS)
        self.assertIn('$crate::dump::Dump::print_to_string', result)
        self.assertIn('$crate::dump::Mode::Full', result)

    def test_proc_macro_attribute_changes_without_changing_wire_header(self):
        result = RENAME.rewrite('crates/vize_davinci_derive/src/model.rs',
            'attr.path().is_ident("folio"); #[folio(name = "s3-folio")] struct L3Folio {}', TARGETS)
        self.assertIn('is_ident("dump")', result)
        self.assertIn('#[dump(name = "s3-folio")]', result)


if __name__ == '__main__':
    unittest.main()
