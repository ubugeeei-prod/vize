//! Fail-closed WIT resolution tests.

use super::*;

fn fixture_resolve() -> (Resolve, PackageId) {
    let mut resolve = Resolve::default();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contracts/base");
    let (package_id, _) = resolve
        .push_dir(&dir)
        .expect("the committed WIT fixture resolves");
    (resolve, package_id)
}

#[test]
fn invalid_package_member_ids_return_errors_instead_of_partial_surfaces() {
    let (mut resolve, package_id) = fixture_resolve();
    let missing = resolve.interfaces.next_id();
    resolve
        .packages
        .get_mut(package_id)
        .expect("the fixture package exists")
        .interfaces
        .insert("missing-interface".into(), missing);
    let error = surface_from_resolve(&resolve, package_id, &Protocol::default())
        .expect_err("an unknown interface must fail the whole surface");
    assert_eq!(error.0, "wit-parser returned an unknown interface id");

    let (mut resolve, package_id) = fixture_resolve();
    let missing = resolve.worlds.next_id();
    resolve
        .packages
        .get_mut(package_id)
        .expect("the fixture package exists")
        .worlds
        .insert("missing-world".into(), missing);
    let error = surface_from_resolve(&resolve, package_id, &Protocol::default())
        .expect_err("an unknown world must fail the whole surface");
    assert_eq!(error.0, "wit-parser returned an unknown world id");
}

#[test]
fn invalid_type_ids_return_errors_instead_of_missing_shapes() {
    let (mut resolve, package_id) = fixture_resolve();
    let missing = resolve.types.next_id();
    let interface_id = *resolve
        .packages
        .get(package_id)
        .expect("the fixture package exists")
        .interfaces
        .values()
        .next()
        .expect("the fixture has an interface");
    resolve
        .interfaces
        .get_mut(interface_id)
        .expect("the fixture interface exists")
        .types
        .insert("missing-type".into(), missing);
    let error = surface_from_resolve(&resolve, package_id, &Protocol::default())
        .expect_err("an unknown type must fail the whole surface");
    assert_eq!(error.0, "wit-parser returned an unknown type id");

    let error = Render(&resolve)
        .ty(&Type::Id(missing))
        .expect_err("an unknown referenced type must fail rendering");
    assert_eq!(error.0, "wit-parser returned an unknown type id");
}

#[test]
fn invalid_nested_references_fail_the_whole_surface() {
    let (mut resolve, package_id) = fixture_resolve();
    let missing = resolve.interfaces.next_id();
    let world_id = *resolve
        .packages
        .get(package_id)
        .expect("the fixture package exists")
        .worlds
        .values()
        .next()
        .expect("the fixture has a world");
    let import = resolve
        .worlds
        .get_mut(world_id)
        .expect("the fixture world exists")
        .imports
        .values_mut()
        .find(|item| matches!(item, WorldItem::Interface { .. }))
        .expect("the fixture imports an interface");
    if let WorldItem::Interface { id, .. } = import {
        *id = missing;
    }
    let error = surface_from_resolve(&resolve, package_id, &Protocol::default())
        .expect_err("an unknown imported interface must fail the whole surface");
    assert_eq!(error.0, "wit-parser returned an unknown interface id");

    let (mut resolve, package_id) = fixture_resolve();
    let missing = resolve.types.next_id();
    let interface_id = *resolve
        .packages
        .get(package_id)
        .expect("the fixture package exists")
        .interfaces
        .get("host-log")
        .expect("the fixture has host-log");
    let param = resolve
        .interfaces
        .get_mut(interface_id)
        .expect("host-log exists")
        .functions
        .get_mut("log")
        .expect("host-log has a log function")
        .params
        .first_mut()
        .expect("log has a parameter");
    param.ty = Type::Id(missing);
    let error = surface_from_resolve(&resolve, package_id, &Protocol::default())
        .expect_err("an unknown function type must fail the whole surface");
    assert_eq!(error.0, "wit-parser returned an unknown type id");
}
