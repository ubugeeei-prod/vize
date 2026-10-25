use super::*;
use std::time::Duration;

#[test]
fn disabled_profiling_installs_no_hook_and_custom_hooks_are_preserved() {
    let config = configure(ApiSpawnConfig::new("checker"), RequestOrigin::Patina, false);
    assert!(config.profiler.is_none());
    let config = configure(config, RequestOrigin::Patina, true);
    let original = config.profiler.as_ref().unwrap().clone();
    let config = configure(config, RequestOrigin::Canon, true);
    assert!(Arc::ptr_eq(&original, config.profiler.as_ref().unwrap()));
    let config = configure(config, RequestOrigin::Canon, false);
    assert!(Arc::ptr_eq(&original, config.profiler.as_ref().unwrap()));
}

#[test]
fn all_request_phases_origins_and_transports_preserve_exact_samples() {
    let profiler = Profiler::enabled();
    for (origin, stage) in [
        (RequestOrigin::Canon, "canon"),
        (RequestOrigin::Patina, "patina"),
    ] {
        for transport in ["jsonrpc", "msgpack"] {
            for (phase, key) in [
                (
                    ApiProfilePhase::SerializeParams,
                    "corsa.request.serialize_params",
                ),
                (ApiProfilePhase::Transport, "corsa.request.transport"),
                (
                    ApiProfilePhase::DeserializeResponse,
                    "corsa.request.deserialize_response",
                ),
                (ApiProfilePhase::DecodeBinary, "corsa.request.decode_binary"),
            ] {
                let event = ApiProfileEvent {
                    method: "updateSnapshot".into(),
                    transport: transport.into(),
                    phase,
                    duration: Duration::from_nanos(37),
                };
                for _ in 0..3 {
                    record_event(&profiler, origin, &event);
                }
                let snapshots = profiler.span_snapshot();
                let matching: Vec<_> = snapshots
                    .iter()
                    .filter(|(name, attribution, _)| {
                        *name == key
                            && attribution.stage == Some(stage)
                            && attribution.pass == Some("updateSnapshot")
                            && attribution.block == Some(transport)
                    })
                    .collect();
                assert_eq!(matching.len(), 1);
                let metrics = &matching.first().unwrap().2;
                assert_eq!(metrics.count, 3);
                assert_eq!(metrics.total_duration, Duration::from_nanos(111));
                assert_eq!(metrics.min_duration, Duration::from_nanos(37));
                assert_eq!(metrics.max_duration, Duration::from_nanos(37));
                assert_eq!(metrics.child_duration, Duration::ZERO);
                assert!(
                    profiler.get(key).is_none(),
                    "do not double count in plain buckets"
                );
            }
        }
    }
    assert_eq!(profiler.span_snapshot().len(), 16);
}

#[test]
fn unknown_server_labels_have_bounded_cardinality_and_disabled_recording_is_empty() {
    let profiler = Profiler::new();
    let mut event = ApiProfileEvent {
        method: "untrusted dynamic method".into(),
        transport: "untrusted transport".into(),
        phase: ApiProfilePhase::Transport,
        duration: Duration::ZERO,
    };
    record_event(&profiler, RequestOrigin::Patina, &event);
    assert!(profiler.span_snapshot().is_empty());
    profiler.enable();
    for index in 0..1024 {
        event.method = vize_l0::cstr!("method-{index}").as_str().into();
        event.transport = vize_l0::cstr!("transport-{index}").as_str().into();
        record_event(&profiler, RequestOrigin::Patina, &event);
    }
    let snapshots = profiler.span_snapshot();
    assert_eq!(snapshots.len(), 1);
    let (name, attribution, metrics) = snapshots.first().unwrap();
    assert_eq!(*name, "corsa.request.transport");
    assert_eq!(attribution.pass, Some("other"));
    assert_eq!(attribution.block, Some("other"));
    assert_eq!(metrics.count, 1024);
    assert_eq!(metrics.total_duration, Duration::ZERO);
}
