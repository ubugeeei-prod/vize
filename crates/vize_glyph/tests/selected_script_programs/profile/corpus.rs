//! Counterbalanced requests on original Vue sources, including single-role files.
use super::*;

#[test]
#[ignore = "timings and isolated allocation assertions on a NUL-delimited original Vue manifest"]
fn original_program_requests() {
    alloc::mark_installed();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = fs::read(std::env::var_os("VIZE_SCRIPT_PROGRAMS_SAMPLE").unwrap()).unwrap();
    let names = list.split(|&b| b == 0).filter(|s| !s.is_empty());
    let total = names.clone().count();
    let stride = if total > 512 { total.div_ceil(192) } else { 1 };
    let mut count = 0;
    for raw in names.step_by(stride) {
        let path = core::str::from_utf8(raw).unwrap();
        let data = fs::read(root.join(path)).unwrap();
        let Ok(text) = core::str::from_utf8(&data) else {
            continue;
        };
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, text, options());
        let Ok(view) = owner.admitted() else {
            continue;
        };
        if view.ordinary().is_none() && view.setup().is_none() {
            continue;
        }
        if [view.ordinary(), view.setup()]
            .into_iter()
            .flatten()
            .any(|script| direct(&arena, script).admitted_program().is_none())
        {
            continue;
        }
        count += 1;
        for read in [
            Read::FirstSetup,
            Read::BothPrograms,
            Read::RepeatedSetup,
            Read::Semantic,
            Read::SemanticAndSetup,
        ] {
            if view.setup().is_none()
                && matches!(
                    read,
                    Read::FirstSetup | Read::RepeatedSetup | Read::SemanticAndSetup
                )
            {
                continue;
            }
            let (a, _, left) = window(view, read, false, false);
            let (b, _, right) = window(view, read, true, false);
            assert_eq!(a, b, "{path}");
            let (left, right) = (left.unwrap(), right.unwrap());
            assert!(
                right.calls <= left.calls
                    && right.peak_bytes_over_start <= left.peak_bytes_over_start,
                "{path} {read:?} {left:?}/{right:?}"
            );
            if matches!(read, Read::RepeatedSetup) {
                assert_eq!((right.calls, right.peak_bytes_over_start), (0, 0));
            }
            let mut pairs = Vec::new();
            for i in 0..4 {
                let reverse = i % 2 == 1;
                let a = window(view, read, reverse, true);
                let b = window(view, read, !reverse, true);
                assert_eq!(a.0, b.0, "{path}");
                pairs.push(if reverse { (b.1, a.1) } else { (a.1, b.1) });
            }
            eprintln!(
                "original read={read:?} bytes={} calls={}/{} peak={}/{} pairs={pairs:?} file={path}",
                text.len(),
                left.calls,
                right.calls,
                left.peak_bytes_over_start,
                right.peak_bytes_over_start
            );
        }
    }
    assert!(count > 0);
    eprintln!("profiled {count} original Vue files");
}
