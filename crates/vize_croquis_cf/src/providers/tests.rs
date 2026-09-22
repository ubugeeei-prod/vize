//! The provider contract: the single-writer check's exact errors, the
//! ambient-input manifest, and the project artifact's module resolution.

use super::vue_router::{RouteParams, RouteTree, VueRouterProvider};
use super::{
    AmbientInput, PROJECT_FACTS, PROVIDERS, ProjectSources, Provider, ProviderDesc, ProviderError,
    check_providers, ids, produced,
};
use vize_davinci::fact::{Demand, FactGroup};
use vize_davinci::pass::AnalysisId;

struct FileRoutes;
impl Provider for FileRoutes {
    const NAME: &'static str = "file-routes";
    const INPUTS: &'static [AmbientInput] = &[AmbientInput::Glob("src/pages/**/*.vue")];
    const OUTPUTS: Demand = Demand::NONE.with(RouteParams::ID);
}

fn registered() -> Demand {
    produced(PROJECT_FACTS.producers())
}

#[test]
fn a_second_writer_to_a_route_group_is_rejected_with_its_exact_error() {
    assert_eq!(
        check_providers(&[VueRouterProvider::DESC, FileRoutes::DESC], registered()),
        Err(ProviderError::SecondWriter {
            group: ids::ROUTE_PARAMS,
            owner: "vue-router",
            writer: "file-routes",
        })
    );
    let tree_writer = ProviderDesc {
        outputs: Demand::NONE.with(RouteTree::ID),
        ..FileRoutes::DESC
    };
    assert_eq!(
        check_providers(&[VueRouterProvider::DESC, tree_writer], registered()),
        Err(ProviderError::SecondWriter {
            group: ids::ROUTE_TREE,
            owner: "vue-router",
            writer: "file-routes",
        })
    );
}

#[test]
fn every_other_contract_violation_has_its_exact_error() {
    let no_inputs = ProviderDesc {
        inputs: &[],
        ..FileRoutes::DESC
    };
    assert_eq!(
        check_providers(&[no_inputs], registered()),
        Err(ProviderError::NoInputs {
            provider: "file-routes"
        })
    );
    let no_outputs = ProviderDesc {
        outputs: Demand::NONE,
        ..FileRoutes::DESC
    };
    assert_eq!(
        check_providers(&[no_outputs], registered()),
        Err(ProviderError::NoOutputs {
            provider: "file-routes"
        })
    );
    let stray = AnalysisId::new(47);
    let unproduced = ProviderDesc {
        outputs: Demand::NONE.with(stray),
        ..FileRoutes::DESC
    };
    assert_eq!(
        check_providers(&[unproduced], registered()),
        Err(ProviderError::UnproducedOutput {
            provider: "file-routes",
            group: stray
        })
    );
}

#[test]
fn the_in_tree_registry_owns_both_route_groups_and_declares_its_inputs() {
    assert_eq!(check_providers(PROVIDERS.providers(), registered()), Ok(()));
    assert_eq!(
        registered(),
        Demand::NONE.with(ids::ROUTE_TREE).with(ids::ROUTE_PARAMS)
    );
    assert_eq!(
        PROVIDERS.owner(ids::ROUTE_TREE).map(|p| p.name),
        Some("vue-router")
    );
    assert_eq!(
        PROVIDERS.owner(ids::ROUTE_PARAMS).map(|p| p.name),
        Some("vue-router")
    );
    assert_eq!(PROVIDERS.owner(AnalysisId::new(0)), None);
    // Demanding the params pulls in the tree, whose provider reads scripts.
    assert_eq!(
        PROVIDERS.inputs(Demand::NONE.with(RouteParams::ID)),
        [AmbientInput::ScriptModules]
    );
    assert_eq!(PROVIDERS.inputs(Demand::NONE), []);
    assert_eq!((RouteTree::STRATUM, RouteParams::STRATUM), (0, 1));
}

#[test]
fn relative_imports_resolve_with_extensions_and_index_files() {
    let mut project = ProjectSources::new();
    let router = project.add("./src/router/index.ts", "export {}").unwrap();
    let admin = project.add("src/router/admin.ts", "export {}").unwrap();
    let view = project
        .add("src/views/Home.vue", "<template><p/></template>")
        .unwrap();
    assert_eq!(
        project.add("src/router/index.ts", ""),
        None,
        "a path is added once"
    );
    assert_eq!(project.add("README.md", ""), None, "only scripts and SFCs");
    assert_eq!(project.module(router).path(), "src/router/index.ts");
    assert_eq!(project.resolve(router, "./admin"), Some(admin));
    assert_eq!(project.resolve(router, "./admin.ts"), Some(admin));
    assert_eq!(project.resolve(router, "../views/Home.vue"), Some(view));
    assert_eq!(project.resolve(view, "../router"), Some(router));
    assert_eq!(
        project.resolve(router, "@/router/admin"),
        None,
        "aliases are not guessed"
    );
    assert_eq!(project.resolve(router, "vue-router"), None);
    let template = project.module(view).template().unwrap();
    assert_eq!((template.text, template.offset), ("<p/>", 10));
}
