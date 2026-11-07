use super::*;
use crate::container::{ContainerFormat, Vue};
use crate::dialect::vue2::surface::TextRefusal;
use crate::dialect::vue2::text::TextBoundaryKind;
use crate::embed::syntax::EmbedHole;
use crate::markup::SurfaceParseOptions;
use crate::markup::{SurfaceChild, check_fidelity};
use alloc::format;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span, String};

mod counts;
mod custody;
mod refusals;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V2,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn resolved_descriptor_and_pool_keep_the_established_debug_surface() {
    let allocator = Allocator::default();
    let observation =
        Vue.observe_vue2_descriptor(&allocator, "<template><div/></template>", options());
    let debug = format!("{observation:#?}");
    assert!(!debug.contains("FileDialect"));
    assert!(!debug.contains("syntax: Standard"));

    let pool = observation.into_expression_pool();
    let pool_debug = format!("{pool:#?}");
    assert!(!pool_debug.contains("FileDialect"));
    assert!(!pool_debug.contains("syntax: Standard"));
}
