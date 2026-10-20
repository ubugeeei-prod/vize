use super::*;
use alloc::format;

#[test]
fn allocation_free_template_owner_matches_the_general_original_descriptor() {
    let arena = Allocator::default();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    let tokens = [
        "雪🌸 text",
        "{",
        "{{",
        "{{ x }}",
        "{{ '</template>' }}",
        "<div class='</template>'>",
        "</div>",
        "<template #x>",
        "</template>",
        "<template/>",
        "<!-- </template> -->",
        "<!--",
        "<div x='",
        "<textarea></template></textarea>",
        "<script> '</template>' </script>",
        "<style>.x {}</style>",
        "{{ /}}/.test(x) }}",
        "{{ `x${x}` }}",
    ];
    for first in tokens {
        for second in tokens {
            let source = format!(" \r\n<template>{first}{second}</template>\t");
            let original = observe(&arena, &source, options);
            let fast = observe_template_only(&source);
            let expected = original.admitted().ok().filter(|_| {
                let [block] = original.container().blocks.as_slice() else {
                    return false;
                };
                block.close_tag.is_some_and(|close| {
                    source
                        .get(close.end as usize..)
                        .is_some_and(|tail| tail.trim_ascii().is_empty())
                })
            });
            assert_eq!(fast.is_some(), expected.is_some(), "{source}");
            if let (Some(fast), Some(expected)) = (fast, expected) {
                let block = &original.container().blocks[0];
                assert_eq!(
                    fast.block().span(),
                    expected.template().unwrap().block().span()
                );
                assert_eq!(fast.open_tag(), block.open_tag);
                assert_eq!(Some(fast.close_tag()), block.close_tag);
            }
        }
    }
    for source in [
        "<template lang=html>x</template>",
        "<template>x</template><template>y</template>",
        "<script>x</script><template>y</template>",
        "<template>x</template><!-- tail -->",
        "<template>x</template>extra",
        "<template>x</template></template>",
    ] {
        assert!(observe_template_only(source).is_none(), "{source}");
    }
}
