//! Check the production scanner on all bounded barrier layouts and arbitrary
//! 64-bit preservation masks. The specification is per member and per bit.
mod prefix;

#[kani::proof]
#[kani::unwind(8)]
fn groups_partition_the_plan_and_preserve_exactly_the_common_analyses() {
    let passes: [(bool, u64); 5] = kani::any();
    let length: usize = kani::any();
    kani::assume(length <= passes.len());
    let bit: u8 = kani::any();
    kani::assume(bit < 64);
    let flag = 1u64 << bit;
    let mut start = 0;
    while let Some((len, barrier, preserved)) = prefix::next_group(&passes[start..length], |p| *p) {
        assert!(start < length && len > 0 && len <= length - start);
        let end = start + len;
        assert_eq!(barrier, !passes[start].0);
        if barrier {
            assert_eq!(len, 1);
        }
        let mut all_preserve = true;
        for (fusable, mask) in &passes[start..end] {
            assert_eq!(*fusable, !barrier);
            all_preserve &= mask & flag != 0;
        }
        assert_eq!(preserved & flag != 0, all_preserve);
        if !barrier && end < length {
            assert!(!passes[end].0);
        }
        start = end;
    }
    assert_eq!(start, length);
}
