//! Ecosystem diagnostics share the collector's descriptor and source revision.

use crate::ide::{DiagnosticService, IdeContext};
use crate::server::ServerState;
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_resident::DescriptorStats;

const SOURCE: &str = "<script setup lang=\"ts\">\r\nimport { useRoute } from 'vue-router'\r\nconst route = useRoute()\r\nconst label = '雪😀'; route.params.slug\r\n</script>\r\n<route lang=\"json\">{\"path\":\"/users/:id\"}</route>";

fn state() -> ServerState {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "ecosystem": true, "lint": false, "typecheck": false,
    })));
    state
}

fn open(state: &ServerState, uri: &Url, source: &str) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
}

#[test]
fn resident_ecosystem_diagnostics_keep_authored_ranges_and_retained_snapshots() {
    let state = state();
    let uri = Url::parse("file:///workspace/Route.vue").unwrap();
    open(&state, &uri, SOURCE);
    let original = IdeContext::with_content(&state, &uri, 0, SOURCE.into());
    let first = super::diagnostics(&uri, original.descriptor().unwrap());
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0].range,
        Range::new(Position::new(3, 34), Position::new(3, 38))
    );
    assert_eq!(first[0].severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(first[0].source.as_deref(), Some("vize/ecosystem"));
    assert_eq!(
        first[0].code,
        Some(NumberOrString::String(
            "ecosystem/vue-router-route-param".into()
        ))
    );
    assert!(first[0].message.contains("Available params: id"));
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );

    // The real aggregation path must publish the same warning using that memo.
    let published = DiagnosticService::collect(&state, &uri);
    assert!(published.contains(&first[0]));
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0
        }
    );
    assert_eq!(DiagnosticService::collect(&state, &uri), published);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0
        }
    );

    let edited = SOURCE.replace("/users/:id", "/users/:slug");
    open(&state, &uri, &edited);
    let updated = IdeContext::with_content(&state, &uri, 0, edited.clone());
    assert!(super::diagnostics(&uri, updated.descriptor().unwrap()).is_empty());
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    assert_eq!(
        super::diagnostics(&uri, original.descriptor().unwrap()),
        first
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 0,
            parses: 0
        }
    );
    let clean = self::state();
    open(&clean, &uri, &edited);
    assert_eq!(
        DiagnosticService::collect(&state, &uri),
        DiagnosticService::collect(&clean, &uri)
    );
}

#[test]
fn resident_ecosystem_diagnostics_share_rejection_recovery_and_reopen() {
    let state = state();
    let uri = Url::parse("file:///workspace/Route.vue").unwrap();
    open(&state, &uri, "<template>{{ $route.params.slug }}");
    for parses in [1, 0] {
        let rejected = DiagnosticService::collect(&state, &uri);
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].source.as_deref(), Some("vize/sfc"));
        assert_eq!(
            state.resident.take_stats(),
            DescriptorStats { lookups: 1, parses }
        );
    }
    open(&state, &uri, SOURCE);
    let fixed = DiagnosticService::collect(&state, &uri);
    assert!(
        fixed
            .iter()
            .any(|diag| diag.source.as_deref() == Some("vize/ecosystem"))
    );
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    open(&state, &uri, SOURCE);
    assert_eq!(DiagnosticService::collect(&state, &uri), fixed);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1
        }
    );
}
