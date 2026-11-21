use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box, time::Instant};

#[global_allocator]
static ALLOC: CountingAllocator<System> = CountingAllocator::system();

pub(super) fn measure(name: &str, owner: &NativeTemplateFile<'_>) {
    alloc::mark_installed();
    for emit in [false, true] {
        let run = |shared| {
            let owner = black_box(owner);
            if shared {
                let pair = build_client_server_decisions(owner).unwrap();
                if emit {
                    let _ = black_box((
                        pair.dom().map(dom_product),
                        pair.ssr().map(|server| {
                            ssr::emit_render_analysis::<Recorded>(server)
                                .map(|w| w.finish().into_document())
                        }),
                    ));
                } else {
                    let _ = black_box((pair.dom(), pair.ssr()));
                }
            } else {
                let dom = build_template_dom_decisions(owner.view().unwrap());
                let ssr = build_template_ssr_decisions(owner.view().unwrap());
                if emit {
                    let _ =
                        black_box((dom.as_ref().map(dom_product), ssr.as_ref().map(ssr_product)));
                } else {
                    let _ = black_box((dom.as_ref(), ssr.as_ref()));
                }
            }
        };
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        assert!(
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
            "{name} emit={emit}"
        );
        let time = |shared| {
            let start = Instant::now();
            for _ in 0..8 {
                run(shared);
            }
            start.elapsed().as_nanos() / 8
        };
        let mut pairs = Vec::new();
        for i in 0..35 {
            let reverse = i % 2 == 1;
            let first = time(reverse);
            let second = time(!reverse);
            if i >= 4 {
                pairs.push(if reverse {
                    (second, first)
                } else {
                    (first, second)
                });
            }
        }
        eprintln!(
            "target_cost {name} emit={emit} calls={}/{} peak={}/{} ns={pairs:?}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
    }
}

#[test]
#[ignore = "complete original target analyses and L4 requests on controlled shapes"]
fn actual_products() {
    for roots in [1, 8, 64] {
        for (name, child) in [
            ("static", "<div id='item'><b>one</b><i>two</i></div>"),
            ("literal", "<p title='a &amp; b' lang='ja'>日本語</p>"),
            ("mixed", "<!--safe--><div><br><input disabled></div>"),
        ] {
            let source = vize_l0::cstr!("<template>{}</template>", child.repeat(roots));
            let arena = Allocator::default();
            let selected =
                vize_l1_to_l2::native_file::lower_selected_sfc_native(&arena, &source, options());
            let owner = selected.template().unwrap();
            assert!(
                owner.view().is_ok(),
                "{name}-{roots}: {:?}",
                selected.issues()
            );
            compare(owner, &mut Counts::default());
            if roots == 64 {
                assert!(
                    owner
                        .view()
                        .unwrap()
                        .file()
                        .unwrap()
                        .artifact()
                        .node_count()
                        > 64
                );
            }
            measure(&vize_l0::cstr!("{name}-{roots}"), owner);
        }
    }
}
