use super::*;
use crate::{
    Backend, BackendFamily, CapabilityDefinition, Environment, Protocol, ProtocolFamily,
    RenderingMode, Route, RuntimeFamily, Target,
};

#[test]
fn accepts_a_cross_language_web_contract() {
    let mut contract = ApplicationContract::new("shop");
    contract.targets.insert(Target::Web);
    contract.environments.push(Environment::new(
        "client",
        Target::Web,
        EnvironmentConsumer::Client,
        RuntimeFamily::Browser,
    ));
    contract.environments.push(Environment::new(
        "server",
        Target::Web,
        EnvironmentConsumer::Server,
        RuntimeFamily::Rust,
    ));
    contract
        .backends
        .push(Backend::new("api", BackendFamily::Rust).with_environment("server"));
    contract.protocols.push(Protocol::new(
        "api.query",
        ProtocolFamily::SchemaQuery,
        "api",
    ));
    let mut route = Route::new("home", "/", "client", RenderingMode::Hybrid);
    route.backend = Some("api".into());
    route.protocol = Some("api.query".into());
    contract.routes.push(route);

    assert_eq!(validate_contract(&contract), Vec::new());
}

#[test]
fn reports_cycles_and_cross_reference_errors() {
    let mut contract = ApplicationContract::new("Broken App");
    contract.targets.insert(Target::Web);
    let mut first = Environment::new(
        "first",
        Target::Web,
        EnvironmentConsumer::Server,
        RuntimeFamily::JavaScript,
    );
    first.depends_on.insert("second".into());
    let mut second = Environment::new(
        "second",
        Target::Web,
        EnvironmentConsumer::Server,
        RuntimeFamily::JavaScript,
    );
    second.depends_on.insert("first".into());
    contract.environments.extend([first, second]);
    contract.routes.push(Route::new(
        "home",
        "missing-slash",
        "missing",
        RenderingMode::Native,
    ));

    let codes = validate_contract(&contract)
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<BTreeSet<_>>();
    assert!(codes.contains("VIZE_MARQUETTE_002"));
    assert!(codes.contains("VIZE_MARQUETTE_014"));
    assert!(codes.contains("VIZE_MARQUETTE_015"));
    assert!(codes.contains("VIZE_MARQUETTE_022"));
}

#[test]
fn rejects_empty_capability_descriptions_and_explains_duplicate_routes() {
    let mut contract = ApplicationContract::new("shop");
    contract.targets.insert(Target::Web);
    contract.environments.push(Environment::new(
        "browser",
        Target::Web,
        EnvironmentConsumer::Client,
        RuntimeFamily::Browser,
    ));
    contract.capabilities.insert(
        "auth.session".into(),
        CapabilityDefinition::new("auth.session", "   "),
    );
    contract.routes.extend([
        Route::new("home", "/", "browser", RenderingMode::Client),
        Route::new("landing", "/", "browser", RenderingMode::Client),
    ]);

    let diagnostics = validate_contract(&contract);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "VIZE_MARQUETTE_024"
            && diagnostic.message == "capability description must not be empty"
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "VIZE_MARQUETTE_019"
            && diagnostic.message == "route path is already used by route \"home\""
    }));
}
