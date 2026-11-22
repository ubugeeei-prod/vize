//! Compare the complete generated setup output with the existing CSS reader.
#![expect(
    clippy::unwrap_used,
    reason = "fixtures must be admitted by both original readers"
)]
use super::collect_css_var_referenced_names;
use crate::virtual_ts::{
    StyleBindingReferences, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_atelier_sfc::{croquis::analyze_sfc_descriptor, parse_sfc};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::{DescriptorOptions, Vue};

#[test]
fn css_reads_preserve_unused_setup_diagnostics_and_all_virtual_mappings() {
    let source = r#"<script setup lang='ts'>const fg='red', size=2, unused=3</script><template><i/></template><style lang='scss'>i{color:v-bind(fg);width:v-bind("size + 'px'")}</style><style lang='less'>.a{color:v-bind(fg)}</style>"#;
    let arena = Allocator::new();
    let original = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let data =
        StyleBindingReferences::observe(&arena, original.admitted().unwrap(), |_| {}).unwrap();
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let names = collect_css_var_referenced_names(&descriptor);
    assert_eq!(data.names(), &names);
    assert_eq!(data.expressions(), ["fg", "size + 'px'"]);
    let template = descriptor.template.as_ref().unwrap();
    let (ast, _) = vize_armature::parse(&arena, &template.content);
    let summary = analyze_sfc_descriptor(&descriptor, Some(&ast), Default::default());
    let script = descriptor.script_setup.as_ref().unwrap();
    let options = VirtualTsOptions::default();
    let old = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(&script.content),
        Some(&ast),
        script.loc.start as u32,
        template.loc.start as u32,
        &options,
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            extra_template_referenced_names: Some(&names),
            ..Default::default()
        },
    );
    let current = data.generate_setup_virtual_ts(
        &summary,
        Some(&script.content),
        Some(&ast),
        script.loc.start as u32,
        template.loc.start as u32,
        &options,
    );
    assert_eq!(current.code, old.code);
    assert_eq!(current.mapping, old.mapping);
    let mut css_only: Vec<_> = data.names().iter().map(|name| name.as_str()).collect();
    css_only.sort_unstable();
    assert_eq!(css_only, ["fg", "size"]);
}
