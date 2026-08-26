use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use vize_s0::String;

use crate::{ContractDiagnostic, validate_contract};

/// Current serialized application-contract format.
///
/// Readers must reject a higher value until they explicitly support it.
pub const CONTRACT_FORMAT_VERSION: u32 = 1;

/// User-visible application target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    /// Standards-based web output.
    Web,
    /// Native mobile or device application output.
    Native,
    /// Desktop application output.
    Desktop,
    /// Terminal application output.
    Terminal,
}

/// Runtime that executes one environment or backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeFamily {
    /// Web browser runtime.
    Browser,
    /// JavaScript server or worker runtime.
    #[serde(rename = "javascript")]
    JavaScript,
    /// Rust executable or library runtime.
    Rust,
    /// Go executable runtime.
    Go,
    /// JVM-family runtime.
    Jvm,
    /// Native device runtime.
    Native,
    /// Desktop host runtime.
    Desktop,
    /// Terminal host runtime.
    Terminal,
    /// Runtime owned by an external adapter.
    External,
}

/// Side of the application graph that consumes an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvironmentConsumer {
    /// Code and assets delivered to an end-user runtime.
    Client,
    /// Trusted server, worker, build, or tool execution.
    Server,
}

/// Backend implementation family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendFamily {
    /// Backend hosted by the JavaScript application runtime.
    #[serde(rename = "javascript")]
    JavaScript,
    /// Rust application or service.
    Rust,
    /// Go application or service.
    Go,
    /// JVM-family application or service.
    Jvm,
    /// Independently managed backend.
    External,
}

/// Supported data and procedure protocol family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolFamily {
    /// Schema-query operations and subscriptions.
    SchemaQuery,
    /// TypeScript-inferred procedures and streams.
    TypedRpc,
    /// Interface-definition binary procedures and streams.
    BinaryRpc,
}

/// Rendering strategy selected for one route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderingMode {
    /// Client-rendered application route.
    Client,
    /// Pre-rendered static route.
    Static,
    /// Server-rendered route.
    Server,
    /// Server-rendered route with streamed output.
    Stream,
    /// Route with explicit partial hydration boundaries.
    Partial,
    /// Route whose output is selected by deployment or request policy.
    Hybrid,
    /// Native host route or screen.
    Native,
    /// Desktop host route or window.
    Desktop,
    /// Terminal route or view.
    Terminal,
}

/// Declares one versioned capability understood by adapters and tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDefinition {
    /// Stable capability identifier.
    pub id: String,
    /// Human-readable contract summary shown by diagnostics and AI tools.
    pub description: String,
    /// Version of the capability contract.
    ///
    /// Defaults to `1`.
    #[serde(default = "default_capability_version")]
    pub version: u32,
}

const fn default_capability_version() -> u32 {
    1
}

impl CapabilityDefinition {
    /// Creates a version-one capability definition.
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            version: default_capability_version(),
        }
    }
}

/// One independently transformed and cached application environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Environment {
    /// Stable environment identifier, unique within the application.
    pub id: String,
    /// User-visible target served by this environment.
    pub target: Target,
    /// Whether the output is consumed by an end-user or trusted runtime.
    pub consumer: EnvironmentConsumer,
    /// Runtime that executes the output.
    pub runtime: RuntimeFamily,
    /// Optional authored entry module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    /// Other environments that must build before this environment.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub depends_on: BTreeSet<String>,
    /// Capabilities required from the environment adapter.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<String>,
}

impl Environment {
    /// Creates an environment without an entry, dependency, or capability.
    pub fn new(
        id: impl Into<String>,
        target: Target,
        consumer: EnvironmentConsumer,
        runtime: RuntimeFamily,
    ) -> Self {
        Self {
            id: id.into(),
            target,
            consumer,
            runtime,
            entry: None,
            depends_on: BTreeSet::new(),
            capabilities: BTreeSet::new(),
        }
    }
}

/// One backend application or independently deployed service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Backend {
    /// Stable backend identifier.
    pub id: String,
    /// Backend implementation family.
    pub family: BackendFamily,
    /// Optional Vize environment that owns local execution for this backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Capabilities required from the backend adapter.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<String>,
}

impl Backend {
    /// Creates a backend without additional capability requirements.
    pub fn new(id: impl Into<String>, family: BackendFamily) -> Self {
        Self {
            id: id.into(),
            family,
            environment: None,
            capabilities: BTreeSet::new(),
        }
    }

    /// Assigns the Vize server environment that owns local execution.
    #[must_use]
    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }
}

/// One protocol endpoint exposed by a backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Protocol {
    /// Stable protocol endpoint identifier.
    pub id: String,
    /// Protocol semantic family.
    pub family: ProtocolFamily,
    /// Backend that owns this endpoint.
    pub backend: String,
    /// Capabilities required from the protocol adapter.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<String>,
}

impl Protocol {
    /// Creates a protocol endpoint without additional capability requirements.
    pub fn new(id: impl Into<String>, family: ProtocolFamily, backend: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            family,
            backend: backend.into(),
            capabilities: BTreeSet::new(),
        }
    }
}

/// One typed application route or native view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Route {
    /// Stable route identifier used by generated navigation APIs.
    pub id: String,
    /// Authored route path. Web paths start with `/`; non-web targets may use a
    /// stable logical path using the same syntax.
    pub path: String,
    /// Environment that owns the rendered route.
    pub environment: String,
    /// Rendering strategy.
    pub rendering: RenderingMode,
    /// Optional backend used by loaders, actions, or server rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Optional protocol endpoint used as the route data contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// Capabilities required by the route.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub capabilities: BTreeSet<String>,
}

impl Route {
    /// Creates a route without backend, protocol, or capability requirements.
    pub fn new(
        id: impl Into<String>,
        path: impl Into<String>,
        environment: impl Into<String>,
        rendering: RenderingMode,
    ) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            environment: environment.into(),
            rendering,
            backend: None,
            protocol: None,
            capabilities: BTreeSet::new(),
        }
    }
}

/// Complete, versioned marquette for a Vize application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplicationContract {
    /// Serialized contract format version.
    ///
    /// Defaults to [`CONTRACT_FORMAT_VERSION`].
    #[serde(default = "default_contract_format_version")]
    pub format_version: u32,
    /// Stable application identifier.
    pub application: String,
    /// User-visible targets included in the application.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub targets: BTreeSet<Target>,
    /// Capability definitions referenced elsewhere in the graph.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub capabilities: BTreeMap<String, CapabilityDefinition>,
    /// Independently transformed application environments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<Environment>,
    /// Backend applications and services.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backends: Vec<Backend>,
    /// Protocol endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protocols: Vec<Protocol>,
    /// Typed application routes and views.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<Route>,
}

const fn default_contract_format_version() -> u32 {
    CONTRACT_FORMAT_VERSION
}

impl ApplicationContract {
    /// Creates an empty contract at the current format version.
    pub fn new(application: impl Into<String>) -> Self {
        Self {
            format_version: CONTRACT_FORMAT_VERSION,
            application: application.into(),
            targets: BTreeSet::new(),
            capabilities: BTreeMap::new(),
            environments: Vec::new(),
            backends: Vec::new(),
            protocols: Vec::new(),
            routes: Vec::new(),
        }
    }

    /// Returns all structural and reference diagnostics in stable path order.
    pub fn validate(&self) -> Vec<ContractDiagnostic> {
        validate_contract(self)
    }
}
