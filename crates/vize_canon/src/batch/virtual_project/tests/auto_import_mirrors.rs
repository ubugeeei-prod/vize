#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::fs;

use crate::virtual_ts::VirtualTsOptions;

use super::super::{AUTO_IMPORT_STUBS_FILE, VirtualProject};
use super::unique_case_dir;

/// Nuxt's `components.d.ts` can type a global component by a script barrel.
/// Its stub must load the mirror, whose `.vue` imports are rewritten; the
/// original reports TS2307 for `./VFInput.vue`, which vue-tsc never does.
#[test]
fn global_component_stubs_load_mirrored_script_modules() {
    let case_dir = unique_case_dir("auto-import-mirrors");
    let _ = fs::remove_dir_all(&case_dir);
    let form_dir = case_dir.join("app/components/form");
    fs::create_dir_all(&form_dir).unwrap();
    fs::write(
        form_dir.join("VFInput.vue"),
        "<script setup lang=\"ts\">defineProps<{ label?: string }>()</script>",
    )
    .unwrap();
    let barrel = form_dir.join("index.ts");
    fs::write(
        &barrel,
        "export { default as VFInput } from \"./VFInput.vue\";\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    let root = project.project_root().to_path_buf();
    let mut options = VirtualTsOptions::default();
    for stub in [
        "declare const Form: (typeof import('{root}/app/components/form/index'))['default'];",
        "declare const Unmirrored: (typeof import('{root}/app/components/form/missing'))['default'];",
        "declare const External: (typeof import('{root}/node_modules/ui/index'))['default'];",
    ] {
        options.auto_import_stubs.push(
            stub.replace("{root}", &root.to_string_lossy())
                .as_str()
                .into(),
        );
    }
    project.set_virtual_ts_options(options);
    project.register_path(&barrel).unwrap();
    project.materialize().unwrap();

    let stubs = fs::read_to_string(project.virtual_root().join(AUTO_IMPORT_STUBS_FILE)).unwrap();
    let root = root.to_string_lossy();
    assert_eq!(
        stubs,
        format!(
            "// @ts-nocheck\n// Framework-provided globals for the virtual project.\n\
             declare const Form: (typeof import('./app/components/form/index'))['default'];\n\
             declare const Unmirrored: (typeof import('{root}/app/components/form/missing'))['default'];\n\
             declare const External: (typeof import('{root}/node_modules/ui/index'))['default'];\n"
        )
    );

    let _ = fs::remove_dir_all(&case_dir);
}
