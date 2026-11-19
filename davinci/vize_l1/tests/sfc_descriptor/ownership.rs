#[test]
fn descriptor_adapter_cannot_reintroduce_a_croquis_container_walk() {
    let source = include_str!("../../src/container/vue/compat/parse.rs");
    assert_eq!(source.matches("split_blocks(source").count(), 1);
    assert_eq!(source.matches("memchr(").count(), 0);
    assert_eq!(source.matches("parse_block_fast(").count(), 0);
}
