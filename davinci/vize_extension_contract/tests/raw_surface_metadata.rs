use vize_extension_contract::SurfacePage;
use vize_l0::dump::{Dump, Mode};
use vize_l0::{Allocator, String};
use vize_l1::{SurfaceChild, check_fidelity};

#[test]
fn raw_page_materialization_preserves_wire_and_holes_without_native_policy_privilege() {
    for (source, expected_wire) in [
        (
            "<div v-pre>{{raw}}</div>",
            "[s1]\nbytes=24\n\n[s1.tree]\nelement\n  lt-name 0:0:4\n  attr\n    name 4:5:10\n  gt 10:10:11\n  text 11:11:18\n  close-tag\n    lt-slash-name 18:18:23\n    gt 23:23:24\n\n",
        ),
        (
            "<div v-pre title='unfinished",
            "[s1]\nbytes=28\n\n[s1.tree]\nelement\n  lt-name 0:0:4\n  attr\n    name 4:5:10\n  attr\n    name 10:11:16\n    eq 16:16:17\n    open-quote 17:17:18\n    value 18:18:28\n    close-quote 28:28:28 missing\n  gt 28:28:28 missing\n  close missing\n\n",
        ),
    ] {
        let allocator = Allocator::new();
        let native = vize_l1::markup::parse_component(&allocator, source).unwrap();
        let SurfaceChild::Element(owner) = &native.tree.children[0] else {
            panic!("owner")
        };
        assert!(owner.open.is_verbatim());
        let page = SurfacePage::of(&native.tree);
        let mut wire = String::default();
        page.print(&mut wire, Mode::Full).unwrap();
        assert_eq!(wire, expected_wire);
        let parsed = <SurfacePage as Dump>::parse(&wire).unwrap();
        assert_eq!(parsed, page);
        let raw_allocator = Allocator::new();
        let raw = parsed.materialize(&raw_allocator, source).unwrap();
        let SurfaceChild::Element(raw_owner) = &raw.children[0] else {
            panic!("raw owner")
        };
        assert!(!raw_owner.open.is_verbatim());
        assert_eq!(raw_owner.open.gt.is_missing(), owner.open.gt.is_missing());
        assert_eq!(check_fidelity(&raw), Ok(()));
        assert_eq!(SurfacePage::of(&raw), page);
        let mut rewired = String::default();
        SurfacePage::of(&raw)
            .print(&mut rewired, Mode::Full)
            .unwrap();
        assert_eq!(rewired, wire);
    }
}
