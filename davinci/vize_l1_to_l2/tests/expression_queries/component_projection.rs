//! Materialize the complete authored import product from either reader.
use vize_atelier_core::Parser;
use vize_croquis::{Croquis, Drawer, DrawerOptions, facts::component_identity};
use vize_l0::{Allocator, Span};
use vize_l1_to_l2::native_file::{
    ComponentImportRefusal, NativeSfcObservation, SetupComponentImports,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Site {
    pub(super) tag: String,
    pub(super) name: Span,
    pub(super) module: String,
    pub(super) export: String,
}

pub(super) fn product(
    owner: &NativeSfcObservation<'_>,
) -> Result<Vec<Site>, ComponentImportRefusal> {
    Ok(retained_product(&owner.setup_component_imports()?))
}

pub(super) fn retained_product(imports: &SetupComponentImports<'_, '_>) -> Vec<Site> {
    imports
        .sites()
        .iter()
        .map(|site| Site {
            tag: site.tag().into(),
            name: site.name_span(),
            module: site.module().into(),
            export: site.export().into(),
        })
        .collect()
}

pub(super) fn draw(source: &str) -> (Croquis, u32, usize) {
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let template = descriptor.template.unwrap();
    let arena = Allocator::new();
    let (root, issues) = Parser::new(&arena, &template.content).parse();
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_script_setup(&descriptor.script_setup.unwrap().content);
    drawer.draw_template(&root);
    (drawer.finish(), template.loc.start as u32, issues.len())
}

pub(super) fn original(source: &str, croquis: &Croquis, start: u32) -> Vec<Site> {
    let mut rows = Vec::new();
    for usage in &croquis.component_usages {
        let first = start + usage.start + 1;
        let tag = source[first as usize..]
            .split(|c: char| c.is_ascii_whitespace() || matches!(c, '>' | '/'))
            .next()
            .unwrap();
        let identity = component_identity(croquis, &usage.name);
        rows.push(Site {
            tag: tag.into(),
            name: Span::new(first, first + tag.len() as u32),
            module: identity.module.unwrap_or_default().as_str().into(),
            export: identity.export_name.as_str().into(),
        });
    }
    rows.sort_unstable_by_key(|site| site.name.start);
    rows
}
