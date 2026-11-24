//! A shared original template for the inspector and the project health report.
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis_cf::TemplateComplexity;
use vize_l0::{
    Allocator, String, cstr,
    pass::{RemarkCollector, observer::RecordedRemark},
};

use super::{InspectorSourceFile, StagePage, StageRemark, stage_feed_value_with_remarks};

/// Build the inspector's serializable stage feed and visit Doctor's component
/// record for each input in order. Inspector remarks stay relative to the
/// template page. Doctor's contributors point into the full original file.
pub fn inspect_template_products(
    files: &[InspectorSourceFile],
    mut component: impl FnMut(&InspectorSourceFile, Option<TemplateComplexity>),
) -> serde_json::Value {
    let mut pages = Vec::new();
    let mut remarks = Vec::new();
    for file in files {
        let complexity = template_products(file).and_then(|products| {
            pages.push(products.page);
            remarks.extend(products.remarks.into_iter().map(|remark| StageRemark {
                path: Some(file.path.clone()),
                remark,
            }));
            products.complexity
        });
        component(file, complexity);
    }
    stage_feed_value_with_remarks("inspector", pages, remarks)
}

struct TemplateProducts {
    page: StagePage,
    remarks: Vec<RecordedRemark>,
    complexity: Option<TemplateComplexity>,
}

fn template_products(file: &InspectorSourceFile) -> Option<TemplateProducts> {
    if !file.path.ends_with(".vue") {
        return None;
    }
    let source = file.source.as_str();
    // The feed and Doctor own their results. Release the SFC descriptor,
    // syntax arena and pass tables before allocating their published rows.
    let (text, remarks, facts) = {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
        let template = descriptor.template?;
        let arena = Allocator::default();
        let (tree, errors) = vize_l1::markup::parse(&arena, template.content.as_ref());
        let mut text = String::with_capacity(template.content.len());
        vize_l1::markup::htmlish::render::render(&tree, &mut |slice| text.push_str(slice));
        let mut observer = RemarkCollector::new();
        let facts = if template.src.is_none()
            && vize_l1::container::lang::is_html_template(template.lang.as_deref())
        {
            let mut lowered = vize_l1_to_l2::lower(&arena, &tree, &errors);
            let facts = vize_l1_to_l2::pass::run_transform(&mut lowered, &mut observer);
            facts.complexity.and_then(|complexity| {
                u32::try_from(template.loc.end).ok()?;
                Some(complexity.shifted(u32::try_from(template.loc.start).ok()?))
            })
        } else {
            None
        };
        (text, observer.finish(), facts)
    };
    let page = StagePage {
        path: Some(file.path.clone()),
        stage: cstr!("s1"),
        pass: cstr!("parse"),
        text,
    };
    let component = facts
        .as_ref()
        .map(|facts| TemplateComplexity::from_facts(facts, source));
    Some(TemplateProducts {
        page,
        remarks,
        complexity: component,
    })
}

#[cfg(test)]
mod tests;
