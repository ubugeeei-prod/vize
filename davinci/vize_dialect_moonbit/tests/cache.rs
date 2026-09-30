//! Reuse and invalidation are observed through actual checker call counts.

use vize_dialect_moonbit::cache::CachedMoonc;
use vize_dialect_moonbit::host::{CheckUnit, HostError, MooncHost, RawCheck};
use vize_l0::String;

#[derive(Debug)]
struct Counter {
    version: String,
    calls: usize,
    fail: bool,
    dependencies: String,
    mutate_dependencies: bool,
    wrong_version: bool,
}

impl MooncHost for Counter {
    fn toolchain(&self) -> &str {
        &self.version
    }
    fn dependency_key(&self) -> Result<String, HostError> {
        Ok(self.dependencies.clone())
    }
    fn check(&mut self, _unit: &CheckUnit<'_>) -> Result<RawCheck, HostError> {
        self.calls += 1;
        if self.mutate_dependencies {
            self.dependencies.push('x');
        }
        if self.fail {
            return Err(HostError::Failed("transport failed".into()));
        }
        Ok(RawCheck {
            toolchain: if self.wrong_version {
                "unexpected".into()
            } else {
                self.version.clone()
            },
            lines: vec!["one diagnostic".into()],
        })
    }
}

fn cache() -> CachedMoonc<Counter> {
    CachedMoonc::new(Counter {
        version: "first".into(),
        calls: 0,
        fail: false,
        dependencies: "core-first".into(),
        mutate_dependencies: false,
        wrong_version: false,
    })
}

fn unit() -> CheckUnit<'static> {
    CheckUnit {
        package: "vize/sfc",
        file_name: "App.vue.mbt",
        source: "let value = 1",
        environment: None,
    }
}

#[test]
fn source_package_file_and_toolchain_each_invalidate_the_result() {
    let mut cache = cache();
    let original = unit();
    let expected = RawCheck {
        toolchain: "first".into(),
        lines: vec!["one diagnostic".into()],
    };
    assert_eq!(cache.check(&original), Ok(expected.clone()));
    assert_eq!(cache.check(&original), Ok(expected));
    assert_eq!(cache.inner_mut().calls, 1);
    for changed in [
        CheckUnit {
            source: "let value = 2",
            ..original
        },
        CheckUnit {
            package: "other/package",
            ..original
        },
        CheckUnit {
            file_name: "Other.vue.mbt",
            ..original
        },
    ] {
        assert_eq!(
            cache.check(&changed).unwrap().toolchain,
            String::from("first")
        );
    }
    assert_eq!(cache.inner_mut().calls, 4);
    cache.inner_mut().version = "second".into();
    assert_eq!(
        cache.check(&original).unwrap().toolchain,
        String::from("second")
    );
    assert_eq!(
        cache.check(&original).unwrap().toolchain,
        String::from("second")
    );
    assert_eq!(cache.inner_mut().calls, 5);
}

#[test]
fn failures_and_inconsistent_toolchain_answers_are_never_cached() {
    let mut cache = cache();
    cache.inner_mut().fail = true;
    for _ in 0..2 {
        assert_eq!(
            cache.check(&unit()),
            Err(HostError::Failed("transport failed".into()))
        );
    }
    cache.inner_mut().fail = false;
    cache.inner_mut().wrong_version = true;
    for _ in 0..2 {
        assert_eq!(
            cache.check(&unit()),
            Err(HostError::Failed(
                "checker answer changed its toolchain during the request".into()
            ))
        );
    }
    cache.inner_mut().wrong_version = false;
    assert_eq!(
        cache.check(&unit()).unwrap().toolchain,
        String::from("first")
    );
    assert_eq!(
        cache.check(&unit()).unwrap().toolchain,
        String::from("first")
    );
    assert_eq!(cache.inner_mut().calls, 5);
}

#[test]
fn entry_and_byte_budgets_evict_or_skip_without_changing_answers() {
    let mut cache = cache();
    for i in 0..33 {
        let source = vize_l0::cstr!("let value = {i}");
        assert_eq!(
            cache
                .check(&CheckUnit {
                    source: &source,
                    ..unit()
                })
                .unwrap()
                .lines,
            vec![String::from("one diagnostic")]
        );
    }
    cache
        .check(&CheckUnit {
            source: "let value = 0",
            ..unit()
        })
        .unwrap();
    // value=0 was evicted by the 33rd distinct entry.
    assert_eq!(cache.inner_mut().calls, 34);
    let source = "x".repeat(4 * 1024 * 1024 + 1);
    for _ in 0..2 {
        cache
            .check(&CheckUnit {
                source: &source,
                ..unit()
            })
            .unwrap();
    }
    assert_eq!(cache.inner_mut().calls, 36);
}

#[test]
fn typed_interface_and_dependency_contents_invalidate_identical_source() {
    let mut cache = cache();
    let original = unit();
    cache.check(&original).unwrap();
    let typed = CheckUnit {
        environment: Some("package \"vize/environment\"\npub fn scope(Bool) -> Unit\n"),
        ..original
    };
    cache.check(&typed).unwrap();
    cache.check(&typed).unwrap();
    let changed = CheckUnit {
        environment: Some("package \"vize/environment\"\npub fn scope(Int) -> Unit\n"),
        ..original
    };
    cache.check(&changed).unwrap();
    assert_eq!(cache.inner_mut().calls, 3);
    cache.inner_mut().dependencies = "core-second".into();
    cache.check(&changed).unwrap();
    assert_eq!(cache.inner_mut().calls, 4);
    let oversized = "x".repeat(4 * 1024 * 1024 + 1);
    for _ in 0..2 {
        cache
            .check(&CheckUnit {
                environment: Some(&oversized),
                ..original
            })
            .unwrap();
    }
    assert_eq!(cache.inner_mut().calls, 6);
}

#[test]
fn dependencies_changing_during_a_check_are_refused_and_retried() {
    let mut cache = cache();
    cache.inner_mut().mutate_dependencies = true;
    for _ in 0..2 {
        assert_eq!(
            cache.check(&unit()),
            Err(HostError::Failed(
                "checker dependencies changed during the request".into()
            ))
        );
    }
    assert_eq!(cache.inner_mut().calls, 2);
}
