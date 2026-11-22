//! Open each original input once for both the correctness and profiling targets.
use std::{fs, path::Path};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::{AdmittedDescriptor, DescriptorOptions, Vue};
pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}

pub fn originals(
    mut inspect: impl for<'a> FnMut(&'a Allocator, &'a str, Option<AdmittedDescriptor<'_, 'a>>, &str),
) -> (usize, usize) {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_STYLE_BINDING_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut inputs, mut invalid_utf8) = (0, 0);
    for path in manifest.split_terminator('\0') {
        inputs += 1;
        let data = fs::read(root.join(path)).unwrap();
        let Ok(source) = std::str::from_utf8(&data) else {
            invalid_utf8 += 1;
            continue;
        };
        let arena = Allocator::new();
        let desc = Vue.observe_descriptor(&arena, source, options());
        let owner = desc.admitted().ok();
        inspect(&arena, source, owner, path);
    }
    (inputs, invalid_utf8)
}
