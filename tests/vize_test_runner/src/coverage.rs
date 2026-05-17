#![allow(clippy::disallowed_macros)]
//! Coverage report generator for Vue compiler tests
//!
//! Usage:
//!   cargo run -p vize_test_runner --bin coverage          # Summary only
//!   cargo run -p vize_test_runner --bin coverage -- -v    # Show failing tests
//!   cargo run -p vize_test_runner --bin coverage -- -vv   # Show diffs

use std::path::PathBuf;
use vize_carton::String;
use vize_test_runner::{CompilerMode, run_fixture_tests};

const MIN_VDOM_PASSED: usize = 353;
const MIN_VAPOR_PASSED: usize = 104;
const MIN_SFC_PASSED: usize = 60;
const MIN_TOTAL_PASSED: usize = 517;

// Known v1 alpha fixture debt. CI allows these exact failures so existing gaps
// do not block unrelated work, but any new failure or pass-count regression
// fails the coverage job.
const KNOWN_FAILURES: &[(&str, &str)] = &[
    ("sfc/basic", "script and template"),
    ("sfc/basic", "lang attributes"),
    (
        "sfc/script-setup",
        "script with type definitions and script setup",
    ),
    (
        "sfc/script-setup",
        "script with interfaces before script setup",
    ),
    (
        "sfc/script-setup",
        "defineProps type-only with destructure defaults",
    ),
    ("sfc/script-setup", "multiline const with type annotation"),
    ("sfc/script-setup", "generic component basic"),
    ("sfc/script-setup", "generic component with extends"),
    (
        "sfc/script-setup",
        "generic component with multiple type params",
    ),
    (
        "sfc/script-setup",
        "generic component with complex constraint",
    ),
    ("sfc/script-setup", "generic component with default type"),
    ("sfc/script-setup", "props with nested object types"),
    ("sfc/script-setup", "props with arrow function types"),
    ("sfc/script-setup", "props with union types"),
    ("sfc/script-setup", "props with intersection types"),
    ("sfc/script-setup", "props with readonly arrays"),
    ("sfc/script-setup", "props with method signatures in object"),
    ("sfc/script-setup", "withDefaults with optional props"),
    ("sfc/script-setup", "withDefaults with function default"),
    ("sfc/script-setup", "multiple top-level await calls"),
    ("sfc/script-setup", "top-level await with destructuring"),
    ("sfc/script-setup", "top-level await in initialization"),
    (
        "sfc/script-setup",
        "defineEmits with typed function signatures",
    ),
    ("sfc/script-setup", "defineEmits with Vue 3.3+ shorthand"),
    ("sfc/script-setup", "interface with callback function type"),
    ("sfc/script-setup", "interface with async callback"),
    ("sfc/script-setup", "array with as const"),
    ("sfc/script-setup", "withDefaults with Object type"),
    ("sfc/script-setup", "computed with route params"),
    ("sfc/script-setup", "reco RoundedBtn pattern"),
    (
        "sfc/script-setup",
        "reco GuidanceProgressLapInputBtn pattern",
    ),
    (
        "sfc/script-setup",
        "nested v-if with ref should not duplicate .value",
    ),
    ("sfc/script-setup", "v-else-if chain with ref bindings"),
    (
        "sfc/script-setup",
        "v-for with imported values should use _unref",
    ),
    ("sfc/script-setup", "v-for with multiple imported values"),
    ("sfc/script-setup", "v-for with imported and local values"),
    (
        "sfc/script-setup",
        "v-model on component with ref binding should not duplicate .value",
    ),
    (
        "sfc/script-setup",
        "v-model with named prop on component with ref binding",
    ),
    (
        "sfc/script-setup",
        "multiple v-model bindings on component with refs",
    ),
    ("sfc/script-setup", "multiline union type definition"),
    ("sfc/script-setup", "multiline intersection type definition"),
    ("sfc/script-setup", "multiline string union type definition"),
    ("sfc/patches", "ES6 shorthand in computed style"),
    ("sfc/patches", "dynamic asset URL with import meta url"),
    ("sfc/patches", "v-if with v-model on input"),
    ("sfc/patches", "v-if with v-model on component"),
    ("sfc/patches", "v-if slot outlet with v-else"),
    ("sfc/patches", "dynamic slot outlet name in loop"),
    (
        "sfc/patches",
        "v for template else interpolation wraps text vnode",
    ),
    (
        "sfc/patches",
        "v if branch keeps maybe ref style patch flag",
    ),
    (
        "sfc/patches",
        "template ref with dynamic props in v if branch",
    ),
    (
        "sfc/patches",
        "options api dynamic style and class keep patch flags",
    ),
    (
        "sfc/patches",
        "imported custom directive binding in script setup",
    ),
    (
        "sfc/patches",
        "component event member expression handler is invoked",
    ),
    ("sfc/patches", "component event rest params stay local"),
    (
        "sfc/patches",
        "function typed prop param does not shadow local t",
    ),
    ("sfc/patches", "nullable runtime prop types keep null"),
    ("sfc/patches", "v-if with custom directive on element"),
    ("sfc/patches", "v-if with component props"),
    ("sfc/patches", "v-if with v-bind object spread"),
    ("sfc/patches", "v-else-if with v-bind object spread"),
    ("sfc/patches", "v-for with v-click-outside"),
    ("sfc/patches", "v-for with custom directive on element"),
    (
        "sfc/patches",
        "props destructure with type-based defineProps and defaults",
    ),
    (
        "sfc/patches",
        "complex type-based props destructure with defaults",
    ),
    ("sfc/patches", "duplicate imports should be filtered"),
    ("sfc/patches", "duplicate named imports should be filtered"),
    ("sfc/patches", "top-level await generates async setup"),
    ("sfc/patches", "type keyword in conditional"),
    ("sfc/patches", "attribute with special characters"),
    (
        "sfc/patches",
        "non-script-setup component with export default",
    ),
    ("sfc/patches", "script with setup function"),
    ("sfc/patches", "generic function call should be stripped"),
    ("sfc/patches", "ref with generic type should be stripped"),
    (
        "sfc/patches",
        "arrow function with typed parameters in template",
    ),
    ("sfc/patches", "callback with typed parameters"),
    (
        "sfc/patches",
        "arrow function with multiple statements in v-for",
    ),
];

fn is_known_failure(path: &str, name: &str) -> bool {
    KNOWN_FAILURES
        .iter()
        .any(|(known_path, known_name)| *known_path == path && *known_name == name)
}

fn main() {
    let args: Vec<String> = std::env::args().map(String::from).collect();
    let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
    let show_diff = args.iter().any(|a| a == "-vv");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest_dir.parent().unwrap().join("fixtures");
    let expected_dir = manifest_dir.parent().unwrap().join("expected");

    let test_files = [
        ("vdom/element", CompilerMode::Vdom),
        ("vdom/component", CompilerMode::Vdom),
        ("vdom/directives", CompilerMode::Vdom),
        ("vdom/hoisting", CompilerMode::Vdom),
        ("vdom/patch-flags", CompilerMode::Vdom),
        ("vdom/v-if", CompilerMode::Vdom),
        ("vdom/v-for", CompilerMode::Vdom),
        ("vdom/v-bind", CompilerMode::Vdom),
        ("vdom/v-on", CompilerMode::Vdom),
        ("vdom/v-model", CompilerMode::Vdom),
        ("vdom/v-slot", CompilerMode::Vdom),
        ("vdom/v-show", CompilerMode::Vdom),
        ("vdom/v-once", CompilerMode::Vdom),
        ("vapor/element", CompilerMode::Vapor),
        ("vapor/component", CompilerMode::Vapor),
        ("vapor/v-if", CompilerMode::Vapor),
        ("vapor/v-for", CompilerMode::Vapor),
        ("vapor/v-bind", CompilerMode::Vapor),
        ("vapor/v-on", CompilerMode::Vapor),
        ("vapor/v-model", CompilerMode::Vapor),
        ("vapor/v-slot", CompilerMode::Vapor),
        ("vapor/v-show", CompilerMode::Vapor),
        ("vapor/edge-cases", CompilerMode::Vapor),
        ("sfc/basic", CompilerMode::Sfc),
        ("sfc/script-setup", CompilerMode::Sfc),
        ("sfc/patches", CompilerMode::Sfc),
    ];

    println!("Vue Compiler Coverage Report");
    println!("============================\n");

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;

    let mut vdom_passed = 0;
    let mut vdom_total = 0;
    let mut vapor_passed = 0;
    let mut vapor_total = 0;
    let mut sfc_passed = 0;
    let mut sfc_total = 0;
    let mut unexpected_failures: Vec<String> = Vec::new();

    for (path, mode) in &test_files {
        let fixture = fixtures_dir.join(format!("{}.toml", path));
        let expected = expected_dir.join(format!("{}.snap", path));

        let results = run_fixture_tests(&fixture, &expected);

        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results
            .iter()
            .filter(|r| !r.passed && r.error.is_some())
            .count();
        let skipped = results
            .iter()
            .filter(|r| !r.passed && r.error.is_none())
            .count();
        let total = results.len();

        total_passed += passed;
        total_failed += failed;
        total_skipped += skipped;
        for result in results
            .iter()
            .filter(|result| !result.passed && result.error.is_some())
        {
            if !is_known_failure(path, result.name.as_str()) {
                unexpected_failures.push(format!("{}: {}", path, result.name).into());
            }
        }

        match mode {
            CompilerMode::Vdom => {
                vdom_passed += passed;
                vdom_total += total;
            }
            CompilerMode::Vapor => {
                vapor_passed += passed;
                vapor_total += total;
            }
            CompilerMode::Sfc => {
                sfc_passed += passed;
                sfc_total += total;
            }
        }

        let pct = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let status = if passed == total {
            "\x1b[32m✓\x1b[0m"
        } else if passed > 0 {
            "\x1b[33m◐\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };

        println!(
            "{} {:25} {:3}/{:3} ({:5.1}%)",
            status, path, passed, total, pct
        );

        // Show details if verbose
        if (verbose || show_diff) && failed > 0 {
            for result in &results {
                if !result.passed
                    && let Some(ref err) = result.error
                {
                    if show_diff {
                        println!("    \x1b[31m✗\x1b[0m {}", result.name);
                        for line in err.lines().take(5) {
                            println!("      {}", line);
                        }
                    } else if verbose {
                        println!("    \x1b[31m✗\x1b[0m {}", result.name);
                    }
                }
            }
        }
    }

    println!("\n----------------------------");

    let vdom_pct = if vdom_total > 0 {
        (vdom_passed as f64 / vdom_total as f64) * 100.0
    } else {
        0.0
    };
    let vapor_pct = if vapor_total > 0 {
        (vapor_passed as f64 / vapor_total as f64) * 100.0
    } else {
        0.0
    };
    let sfc_pct = if sfc_total > 0 {
        (sfc_passed as f64 / sfc_total as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "VDOM:   {:3}/{:3} ({:5.1}%)",
        vdom_passed, vdom_total, vdom_pct
    );
    println!(
        "Vapor:  {:3}/{:3} ({:5.1}%)",
        vapor_passed, vapor_total, vapor_pct
    );
    println!(
        "SFC:    {:3}/{:3} ({:5.1}%)",
        sfc_passed, sfc_total, sfc_pct
    );

    println!("\n============================");

    let total = total_passed + total_failed + total_skipped;
    let total_pct = if total > 0 {
        (total_passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "TOTAL:  {:3}/{:3} ({:5.1}%)",
        total_passed, total, total_pct
    );

    if total_failed > 0 {
        println!("\n{} tests failed", total_failed);
        println!(
            "{} known failure(s) tracked for v1 alpha",
            KNOWN_FAILURES.len()
        );
    }

    let mut budget_failures = Vec::new();
    if vdom_passed < MIN_VDOM_PASSED {
        budget_failures.push(format!("VDOM passed {} < {}", vdom_passed, MIN_VDOM_PASSED));
    }
    if vapor_passed < MIN_VAPOR_PASSED {
        budget_failures.push(format!(
            "Vapor passed {} < {}",
            vapor_passed, MIN_VAPOR_PASSED
        ));
    }
    if sfc_passed < MIN_SFC_PASSED {
        budget_failures.push(format!("SFC passed {} < {}", sfc_passed, MIN_SFC_PASSED));
    }
    if total_passed < MIN_TOTAL_PASSED {
        budget_failures.push(format!(
            "Total passed {} < {}",
            total_passed, MIN_TOTAL_PASSED
        ));
    }

    if !unexpected_failures.is_empty() {
        println!("\nUnexpected coverage failures:");
        for failure in &unexpected_failures {
            println!("  - {}", failure);
        }
    }
    if !budget_failures.is_empty() {
        println!("\nCoverage budget regressions:");
        for failure in &budget_failures {
            println!("  - {}", failure);
        }
    }

    if !unexpected_failures.is_empty() || !budget_failures.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{KNOWN_FAILURES, is_known_failure};
    use vize_carton::FxHashSet;

    #[test]
    fn tracks_the_current_known_failure_budget() {
        assert_eq!(KNOWN_FAILURES.len(), 77);
        let unique_failures: FxHashSet<_> = KNOWN_FAILURES.iter().collect();
        assert_eq!(unique_failures.len(), KNOWN_FAILURES.len());
        assert!(is_known_failure(
            "sfc/script-setup",
            "defineProps type-only with destructure defaults"
        ));
        assert!(!is_known_failure("vapor/v-if", "v-if/v-else-if/v-else"));
        assert!(!is_known_failure("vdom/element", "plain element"));
    }
}
