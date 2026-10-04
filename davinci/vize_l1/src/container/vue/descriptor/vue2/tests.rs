use super::*;
use crate::SurfaceParseOptions;
use crate::container::{ContainerFormat, Vue};
use crate::dialect::vue2::surface::{self, TextRefusal};
use crate::dialect::vue2::text::TextBoundaryKind;
use crate::embed::syntax::EmbedHole;
use crate::{SurfaceChild, check_fidelity};
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
