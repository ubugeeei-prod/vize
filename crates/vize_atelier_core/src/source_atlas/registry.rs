//! Cheap Source Atlas request notes.

use super::{
    SourceAtlasCoordinate, SourceAtlasFallback, SourceAtlasFallbackSet, SourceAtlasPlate,
    SourceAtlasRequest, SourceAtlasRoute, SourceAtlasSource, SourceAtlasTarget,
};

/// Product lane that requested atlas facts.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SourceAtlasLane {
    /// SFC/template/script/style compilation and target code generation.
    Compiler,
    /// Patina linting and rule-specific semantic projections.
    Linter,
    /// Canon/Maestro typechecking and Virtual TS projections.
    Typecheck,
    /// Language server features such as hover, rename, references, and
    /// completions.
    LanguageServer,
    /// Glyph formatting and lexical/source-shape projections.
    Formatter,
    /// Inspector and debug views that expose internal plates on demand.
    Inspector,
    /// Playground and browser-facing preview surfaces.
    Playground,
    /// Source-map registration and composition work.
    SourceMap,
    /// Bundler integration and Vite/Nuxt package surfaces.
    Bundler,
    /// Public binding/package payloads exposed through Vitrine.
    Vitrine,
}

impl SourceAtlasLane {
    /// Counter used when this product lane asks the atlas for facts.
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::Compiler => "atelier.profile.lane.compiler",
            Self::Linter => "atelier.profile.lane.linter",
            Self::Typecheck => "atelier.profile.lane.typecheck",
            Self::LanguageServer => "atelier.profile.lane.language_server",
            Self::Formatter => "atelier.profile.lane.formatter",
            Self::Inspector => "atelier.profile.lane.inspector",
            Self::Playground => "atelier.profile.lane.playground",
            Self::SourceMap => "atelier.profile.lane.source_map",
            Self::Bundler => "atelier.profile.lane.bundler",
            Self::Vitrine => "atelier.profile.lane.vitrine",
        }
    }
}

/// Resolved, copyable request note for a Source Atlas lane.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SourceAtlasRegistry {
    /// Product lane that made the request.
    ///
    /// The lane is recorded separately from sources and targets so Vize can
    /// distinguish "compiler requested source maps" from "source-map
    /// composition lane registered source maps" without inventing target names
    /// for product surfaces.
    pub lane: SourceAtlasLane,
    /// Source, plate, target, and compatibility coordinate requested by the
    /// lane.
    pub route: SourceAtlasRoute,
    /// Compatibility or reduced-projection facts observed while serving the
    /// request.
    pub fallbacks: SourceAtlasFallbackSet,
}

impl SourceAtlasRegistry {
    /// Start a registry note for a product lane with no requested plates yet.
    pub const fn new(lane: SourceAtlasLane) -> Self {
        Self {
            lane,
            route: SourceAtlasRoute::empty(),
            fallbacks: SourceAtlasFallbackSet::empty(),
        }
    }

    /// Start a compiler-lane request.
    pub const fn compiler() -> Self {
        Self::new(SourceAtlasLane::Compiler)
    }

    /// Start a source-map-lane request.
    pub const fn source_map() -> Self {
        Self::new(SourceAtlasLane::SourceMap)
    }

    /// Start a typecheck-lane request.
    pub const fn typecheck() -> Self {
        Self::new(SourceAtlasLane::Typecheck)
    }

    /// Replace the current route with a prebuilt route.
    pub const fn with_route(mut self, route: SourceAtlasRoute) -> Self {
        self.route = route;
        self
    }

    /// Add one source entry to the route.
    pub const fn with_source(mut self, source: SourceAtlasSource) -> Self {
        self.route = self.route.with_source(source);
        self
    }

    /// Add one demanded plate to the route.
    pub const fn with_plate(mut self, plate: SourceAtlasPlate) -> Self {
        self.route = self.route.with_plate(plate);
        self
    }

    /// Add one target product to the route.
    pub const fn with_target(mut self, target: SourceAtlasTarget) -> Self {
        self.route = self.route.with_target(target);
        self
    }

    /// Attach a version or capability coordinate to the route.
    pub const fn with_coordinate(mut self, coordinate: SourceAtlasCoordinate) -> Self {
        self.route = self.route.with_coordinate(coordinate);
        self
    }

    /// Record that serving this registry note used a fallback path.
    pub const fn with_fallback(mut self, fallback: SourceAtlasFallback) -> Self {
        self.fallbacks = self.fallbacks.with(fallback);
        self
    }

    /// Visit the requested source, plate, target, and coordinate facts in
    /// stable profile order.
    pub fn visit_requests(self, mut visit: impl FnMut(SourceAtlasRequest)) {
        for source in self.route.sources.iter() {
            visit(SourceAtlasRequest::Source(source));
        }
        for plate in self.route.plates.iter() {
            visit(SourceAtlasRequest::Plate(plate));
        }
        for target in self.route.targets.iter() {
            visit(SourceAtlasRequest::Target(target));
        }
        if let Some(coordinate) = self.route.coordinate {
            visit(SourceAtlasRequest::Coordinate(coordinate));
        }
    }
}
