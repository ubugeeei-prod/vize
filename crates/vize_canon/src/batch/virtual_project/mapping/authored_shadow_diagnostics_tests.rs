//! An owned authored copy retains its own coordinates, never the generated map.

use std::{fs, path::PathBuf};

use super::VirtualProject;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/package-import-redirects-11317/input.json"
));

fn project(
    plain_source: Option<&str>,
) -> Result<(tempfile::TempDir, VirtualProject, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let input: serde_json::Value = serde_json::from_str(INPUT)?;
    assert_eq!(input["schema"], "vize.package-import-redirects");
    assert_eq!(
        input["id"],
        "typechecker/regression/package-import-redirects-11317"
    );
    for file in input["files"]
        .as_array()
        .ok_or("the five original inputs")?
    {
        let name = file["path"].as_str().ok_or("input path")?;
        let original = file["source"].as_str().ok_or("input source")?;
        let source = if name == "src/plain.ts" {
            plain_source.unwrap_or(original)
        } else {
            original
        };
        let path = root.join(name);
        fs::create_dir_all(path.parent().ok_or("input parent")?)?;
        fs::write(path, source)?;
    }
    let main = root.join("src/main.ts");
    let plain = root.join("src/plain.ts");
    let mut project = VirtualProject::new(&root)?;
    project.register_path(&main)?;
    project.reconcile_package_routes_for_importers(std::slice::from_ref(&main));
    project.register_paths(&[main, plain.clone(), root.join("src/Other.vue")])?;
    project.register_package_route_targets()?;
    project.finalize_package_routes()?;
    let canonical = project
        .find_by_original(&plain)
        .ok_or("registered plain.ts")?
        .virtual_path
        .clone();
    let shadow = project
        .package_shadow_files
        .iter()
        .find_map(|(shadow, owner)| {
            (owner == &canonical
                && shadow != &canonical
                && shadow.file_name().is_some_and(|name| name == "plain.ts"))
            .then_some(shadow.clone())
        })
        .ok_or("the genuine private-import route must own a relocated plain.ts")?;
    Ok((directory, project, plain, shadow))
}

#[test]
fn owned_authored_package_copy_retains_the_original_imported_file_position() {
    let (_root, project, plain, shadow) = project(None).unwrap();
    let file = project.find_by_original(&plain).unwrap();
    let authored = project
        .original_content_for_virtual(&file.virtual_path)
        .unwrap();
    assert_ne!(authored, file.content.as_str());
    assert_eq!(
        project
            .package_shadow_content(&shadow, &file.virtual_path)
            .unwrap()
            .as_str(),
        authored
    );
    assert!(project.find_by_diagnostic_virtual(&shadow).is_none());
    let original = project.diagnostic_position(&shadow, 4, 6).unwrap();
    assert_eq!(
        (
            original.path,
            original.line,
            original.column,
            original.block_type
        ),
        (plain, 4, 6, None)
    );
    assert!(
        project
            .find_by_diagnostic_virtual(&file.virtual_path)
            .is_some()
    );
}

#[test]
fn authored_identity_keeps_utf16_unicode_and_crlf_across_same_line_import_growth() {
    for newline in ["\n", "\r\n"] {
        let source = [
            "// 雪🌸",
            "/* 雪🌸 */ import Other from \"./Other.vue\"; const probe: number = \"bad\";",
            "export const value = 1;",
            "export { Other, probe };",
            "",
        ]
        .join(newline);
        let (_root, project, plain, shadow) = project(Some(&source)).unwrap();
        let file = project.find_by_original(&plain).unwrap();
        let line = source.lines().nth(1).unwrap();
        let byte_column = line.find("probe").unwrap();
        let column = line[..byte_column].encode_utf16().count() as u32;
        assert_ne!(column as usize, byte_column);
        let original = project.diagnostic_position(&shadow, 1, column).unwrap();
        assert_eq!(
            (original.path, original.line, original.column),
            (plain.clone(), 1, column)
        );
        let (canonical, generated_line, generated_column) =
            project.map_to_virtual(&plain, 1, column).unwrap();
        assert_eq!((generated_line, generated_column), (1, column + 3));
        assert_eq!(canonical, file.virtual_path);
        let mapped = project
            .map_to_original(&canonical, generated_line, generated_column)
            .unwrap();
        assert_eq!(
            (mapped.path, mapped.line, mapped.column),
            (plain, 1, column)
        );
        assert!(project.find_by_diagnostic_virtual(&shadow).is_none());
    }
}

#[test]
fn synthetic_foreign_and_missing_snapshot_sources_cannot_borrow_authored_identity() {
    let (_root, mut project, plain, shadow) = project(None).unwrap();
    let canonical = project
        .find_by_original(&plain)
        .unwrap()
        .virtual_path
        .clone();
    let foreign = shadow.with_file_name("unowned.ts");
    let authored = project.original_content_for_virtual(&canonical).unwrap();
    fs::create_dir_all(foreign.parent().unwrap()).unwrap();
    fs::write(&foreign, authored).unwrap();
    assert!(project.diagnostic_position(&foreign, 4, 6).is_none());
    let companion = project
        .package_shadow_files
        .keys()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "Other.d.vue.ts")
        })
        .unwrap();
    assert!(project.find_by_diagnostic_virtual(companion).is_none());
    assert!(project.diagnostic_position(companion, 0, 0).is_none());
    assert!(project.diagnostic_position(&canonical, 4, 6).is_none());
    assert!(project.find_by_diagnostic_virtual(&canonical).is_some());
    project.remove_original_content_for_test(&canonical);
    assert!(project.diagnostic_position(&shadow, 4, 6).is_none());
    project.package_shadow_files.insert(shadow.clone(), foreign);
    assert!(project.diagnostic_position(&shadow, 4, 6).is_none());
}
