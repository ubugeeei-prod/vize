use super::SfcCroquisOptions;
use super::script_demand::{CapturedScript, OrdinaryScript, ScriptDemand};
use crate::types::SfcDescriptor;
use vize_carton::profile;
use vize_croquis::binding_occurrences::BindingOccurrences;
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
    analyze_scripts_impl::<OrdinaryScript>(descriptor, options, options_api, legacy_vue2)
}

pub(super) fn analyze_scripts_with_occurrences(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> (Croquis, Option<BindingOccurrences>) {
    analyze_scripts_impl::<CapturedScript>(descriptor, options, options_api, legacy_vue2)
}

fn analyze_scripts_impl<D: ScriptDemand>(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> D::Output {
    let drawer_options = options.analyzer_options;
    if !drawer_options.analyze_script {
        return D::disabled();
    }
    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) if options.merge_scripts => {
            let plain_drawer = D::prepare(Drawer::with_options(drawer_options));
            let mut plain_drawer = apply_options_api_mode(plain_drawer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                plain_drawer.draw_script_plain_with_jsx(
                    script.content.as_ref(),
                    script_lang_is_jsx(script.lang.as_deref()),
                )
            );
            let plain = D::finish(plain_drawer);
            let setup_drawer = D::prepare(Drawer::with_options(drawer_options));
            let setup_drawer = demand_unused(setup_drawer, options);
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
            let setup = D::finish(setup_drawer);
            let setup_offset = script.content.len() as u32 + 1;
            D::merge(plain, setup, setup_offset)
        }
        (_, Some(script_setup)) => {
            let drawer = D::prepare(Drawer::with_options(drawer_options));
            let drawer = demand_unused(drawer, options);
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
            D::finish(drawer)
        }
        (Some(script), None) => {
            let drawer = D::prepare(Drawer::with_options(drawer_options));
            let mut drawer = apply_options_api_mode(drawer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                drawer.draw_script_plain_with_jsx(
                    script.content.as_ref(),
                    script_lang_is_jsx(script.lang.as_deref()),
                )
            );
            D::finish(drawer)
        }
        (None, None) => D::empty(),
    }
}

fn script_lang_is_jsx(lang: Option<&str>) -> bool {
    matches!(lang.map(str::trim), Some("tsx" | "jsx"))
}

fn demand_unused(drawer: Drawer, options: SfcCroquisOptions) -> Drawer {
    if options.unused_bindings {
        drawer.with_unused_bindings()
    } else {
        drawer
    }
}
