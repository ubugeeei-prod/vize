use super::SfcCroquisOptions;
use crate::types::SfcDescriptor;
use vize_carton::profile;
use vize_croquis::{Croquis, Drawer};

pub(super) fn apply_options_api_mode(
    drawer: Drawer,
    options_api: bool,
    legacy_vue2: bool,
) -> Drawer {
    if legacy_vue2 {
        drawer.with_legacy_vue2()
    } else if options_api {
        drawer.with_options_api()
    } else {
        drawer
    }
}

pub(super) fn analyze_scripts(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> Croquis {
    let drawer_options = options.analyzer_options;
    if !drawer_options.analyze_script {
        return Croquis::new();
    }
    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) if options.merge_scripts => {
            let plain_drawer = Drawer::with_options(drawer_options);
            let mut plain_drawer = apply_options_api_mode(plain_drawer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                plain_drawer.draw_script_plain_with_jsx(
                    script.content.as_ref(),
                    script_lang_is_jsx(script.lang.as_deref()),
                )
            );
            let plain = plain_drawer.finish();

            let setup_drawer = Drawer::with_options(drawer_options);
            let mut setup_drawer = apply_options_api_mode(setup_drawer, options_api, legacy_vue2);
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            profile!(
                "atelier.sfc.croquis.script_setup",
                setup_drawer.draw_script_setup_with_generic_and_jsx(
                    script_setup.content.as_ref(),
                    generic,
                    script_lang_is_jsx(script_setup.lang.as_deref()),
                )
            );

            let mut summary = setup_drawer.finish();
            let setup_offset = script.content.len() as u32 + 1;
            summary.shift_script_offsets(setup_offset);
            summary.merge_plain_script(plain);
            summary
        }
        (_, Some(script_setup)) => {
            let drawer = Drawer::with_options(drawer_options);
            let mut drawer = apply_options_api_mode(drawer, options_api, legacy_vue2);
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            profile!(
                "atelier.sfc.croquis.script_setup",
                drawer.draw_script_setup_with_generic_and_jsx(
                    script_setup.content.as_ref(),
                    generic,
                    script_lang_is_jsx(script_setup.lang.as_deref()),
                )
            );
            drawer.finish()
        }
        (Some(script), None) => {
            let drawer = Drawer::with_options(drawer_options);
            let mut drawer = apply_options_api_mode(drawer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                drawer.draw_script_plain_with_jsx(
                    script.content.as_ref(),
                    script_lang_is_jsx(script.lang.as_deref()),
                )
            );
            drawer.finish()
        }
        (None, None) => Croquis::new(),
    }
}

fn script_lang_is_jsx(lang: Option<&str>) -> bool {
    matches!(lang.map(str::trim), Some("tsx" | "jsx"))
}
