//! Stable Fx digest, map, observer-allocation and thread-count laws.
use rustc_hash::{FxBuildHasher, FxHasher};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

vize_l0::scoped_fx_hasher!(Observed, SourceOwner, "test.scoped_fx");

fn observed() -> Observed {
    Default::default()
}

#[cfg(feature = "micro-profiling")]
#[global_allocator]
static ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =
    vize_l0::profiler::ProfilingAllocator::from_allocator(std::alloc::System);

fn digest_and_map_parity() {
    let mut raw = FxHasher::default();
    let mut observed = observed().build_hasher();
    for bytes in [
        b"".as_slice(),
        b"ab".as_slice(),
        b"c".as_slice(),
        "日本🙂\0".as_bytes(),
    ] {
        raw.write(bytes);
        observed.write(bytes);
        assert_eq!(raw.finish(), observed.finish());
    }
    macro_rules! primitive {
        ($method:ident, $kind:ty) => {
            for value in [<$kind>::MIN, 0, <$kind>::MAX] {
                raw.$method(value);
                observed.$method(value);
                assert_eq!(raw.finish(), observed.finish());
            }
        };
    }
    primitive!(write_u8, u8);
    primitive!(write_u16, u16);
    primitive!(write_u32, u32);
    primitive!(write_u64, u64);
    primitive!(write_u128, u128);
    primitive!(write_usize, usize);
    primitive!(write_i8, i8);
    primitive!(write_i16, i16);
    primitive!(write_i32, i32);
    primitive!(write_i64, i64);
    primitive!(write_i128, i128);
    primitive!(write_isize, isize);
    let raw_clone = raw.clone();
    let observed_clone = observed.clone();
    assert_eq!(raw_clone.finish(), observed_clone.finish());
    assert_eq!(raw.finish(), observed.finish());
    for value in ["", "a", "a\0", "日本🙂", "é", "e\u{301}"] {
        assert_eq!(FxBuildHasher.hash_one(value), observed().hash_one(value));
        assert_eq!(
            FxBuildHasher.hash_one((value, 7_u64)),
            observed().hash_one((value, 7_u64))
        );
    }
    for value in [vec![], vec![0_u8], vec![1, 2, 3, 255]] {
        assert_eq!(FxBuildHasher.hash_one(&value), observed().hash_one(&value));
    }
    let keys = (0..512)
        .map(|i| vize_l0::String::from(format!("key_{i}_日本🙂_long_payload")))
        .collect::<Vec<_>>();
    let mut raw: HashMap<vize_l0::String, usize, FxBuildHasher> = HashMap::default();
    let mut observed: HashMap<vize_l0::String, usize, Observed> = HashMap::default();
    assert_eq!(raw.get("absent"), observed.get("absent"));
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(raw.insert(key.clone(), i), observed.insert(key.clone(), i));
        assert_eq!(raw.capacity(), observed.capacity());
    }
    for key in &keys {
        assert_eq!(raw.get(key.as_str()), observed.get(key.as_str()));
        assert_eq!(
            raw.get_key_value(key.as_str()),
            observed.get_key_value(key.as_str())
        );
    }
    for (i, key) in keys.iter().enumerate().step_by(3) {
        assert_eq!(raw.remove(key.as_str()), observed.remove(key.as_str()));
        assert_eq!(
            raw.insert(key.clone(), i + 1024),
            observed.insert(key.clone(), i + 1024)
        );
    }
    assert_eq!(
        raw.iter().collect::<Vec<_>>(),
        observed.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        raw.clone().into_iter().collect::<Vec<_>>(),
        observed.clone().into_iter().collect::<Vec<_>>()
    );
}

#[cfg(feature = "micro-profiling")]
fn controlled_counts_and_allocations() {
    use vize_l0::profiler::{global_profiler, micro};
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();
    let guard = profiler.global_span("test.scoped_fx.owner");
    let builder = observed();
    let mut hasher = builder.build_hasher();
    hasher.write(b"abc");
    hasher.write_u8(7);
    hasher.write_u16(7);
    hasher.write_u32(7);
    hasher.write_u64(7);
    hasher.write_u128(7);
    hasher.write_usize(7);
    hasher.write_i8(7);
    hasher.write_i16(7);
    hasher.write_i32(7);
    hasher.write_i64(7);
    hasher.write_i128(7);
    hasher.write_isize(7);
    let first = hasher.finish();
    let repeated = hasher.finish();
    let one = builder.hash_one(23_u64);
    drop(guard);
    profiler.disable();
    let rows = micro::snapshot();
    assert_eq!(first, repeated);
    assert_eq!(one, FxBuildHasher.hash_one(23_u64));
    // TLS attribution excludes the libtest reporter and all other threads.
    assert_eq!(
        profiler.get("test.scoped_fx.owner").map(|m| (
            m.count,
            m.alloc_calls,
            m.alloc_bytes,
            m.self_alloc_calls,
            m.self_alloc_bytes
        )),
        Some((1, 0, 0, 0, 0))
    );
    println!(
        "scoped_fx_observer_allocations={}",
        serde_json::json!(
            profiler
                .get("test.scoped_fx.owner")
                .map(|m| serde_json::json!({
                    "count": m.count, "allocCalls": m.alloc_calls, "requestedBytes": m.alloc_bytes,
                    "selfAllocCalls": m.self_alloc_calls, "selfRequestedBytes": m.self_alloc_bytes,
                    "scope": "same Rust caller thread; observer registration excluded"
                }))
        )
    );
    let actual = rows
        .iter()
        .map(|r| (r.name, (r.calls, r.units)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let expected = std::collections::BTreeMap::from([
        ("test.scoped_fx.build", (2, 0)),
        ("test.scoped_fx.hash_one", (1, 0)),
        ("test.scoped_fx.finish", (3, 0)),
        ("test.scoped_fx.write.bytes", (1, 3)),
        (
            "test.scoped_fx.write_u8.bytes",
            (1, std::mem::size_of::<u8>() as u64),
        ),
        (
            "test.scoped_fx.write_u16.bytes",
            (1, std::mem::size_of::<u16>() as u64),
        ),
        (
            "test.scoped_fx.write_u32.bytes",
            (1, std::mem::size_of::<u32>() as u64),
        ),
        ("test.scoped_fx.write_u64.bytes", (2, 16)),
        (
            "test.scoped_fx.write_u128.bytes",
            (1, std::mem::size_of::<u128>() as u64),
        ),
        (
            "test.scoped_fx.write_usize.bytes",
            (1, std::mem::size_of::<usize>() as u64),
        ),
        (
            "test.scoped_fx.write_i8.bytes",
            (1, std::mem::size_of::<i8>() as u64),
        ),
        (
            "test.scoped_fx.write_i16.bytes",
            (1, std::mem::size_of::<i16>() as u64),
        ),
        (
            "test.scoped_fx.write_i32.bytes",
            (1, std::mem::size_of::<i32>() as u64),
        ),
        (
            "test.scoped_fx.write_i64.bytes",
            (1, std::mem::size_of::<i64>() as u64),
        ),
        (
            "test.scoped_fx.write_i128.bytes",
            (1, std::mem::size_of::<i128>() as u64),
        ),
        (
            "test.scoped_fx.write_isize.bytes",
            (1, std::mem::size_of::<isize>() as u64),
        ),
    ]);
    assert_eq!(actual, expected);
    println!("scoped_fx_interfaces={}", serde_json::json!(&rows));
    profiler.clear();

    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    let go = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicUsize::new(0));
    let workers = (0..4)
        .map(|_| {
            let go = go.clone();
            let ready = ready.clone();
            let done = done.clone();
            std::thread::spawn(move || {
                ready.fetch_add(1, Ordering::Release);
                while !go.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                let builder = observed();
                for i in 0..1000_u64 {
                    std::hint::black_box(builder.hash_one(i));
                }
                done.fetch_add(1, Ordering::Release);
            })
        })
        .collect::<Vec<_>>();
    while ready.load(Ordering::Acquire) != 4 {
        std::thread::yield_now();
    }
    profiler.enable();
    go.store(true, Ordering::Release);
    while done.load(Ordering::Acquire) != 4 {
        std::thread::yield_now();
    }
    profiler.disable();
    for worker in workers {
        assert_eq!(worker.join().map_err(|_| "worker panic"), Ok(()));
    }
    let threaded = micro::snapshot();
    let actual = threaded
        .iter()
        .map(|r| (r.name, (r.calls, r.units)))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        actual,
        std::collections::BTreeMap::from([
            ("test.scoped_fx.build", (4000, 0)),
            ("test.scoped_fx.hash_one", (4000, 0)),
            ("test.scoped_fx.finish", (4000, 0)),
            ("test.scoped_fx.write_u64.bytes", (4000, 32000)),
        ])
    );
    println!("scoped_fx_threads={}", serde_json::json!(&threaded));
}

#[test]
fn stable_fx_delegation_preserves_every_interface_and_map_observation() {
    #[cfg(feature = "micro-profiling")]
    vize_l0::profiler::global_profiler().disable();
    #[cfg(not(feature = "micro-profiling"))]
    let _: FxBuildHasher = observed();
    digest_and_map_parity();
    #[cfg(feature = "micro-profiling")]
    controlled_counts_and_allocations();
    println!(
        "scoped_fx_qualified={}",
        serde_json::json!({
            "scope": "explicit owner stable Fx interfaces; no backend/PMU/normal-time qualification",
            "normalAliasExactRawType": !cfg!(feature = "micro-profiling"),
            "profileBuilderIsZeroSized": std::mem::size_of::<Observed>() == 0,
            "unsupportedWriteStrInterfaceCount": null,
            "unsupportedWriteLengthPrefixInterfaceCount": null,
            "completeHashComputationCount": null,
            "normalTiming": null,
        })
    );
}
