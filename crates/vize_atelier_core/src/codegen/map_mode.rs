//! Which render map a generator records, chosen when it is compiled.
//!
//! Every generator records the plain map when a compile asks for one. The
//! SFC assembly composes its render map into the component's, and that map
//! also needs links no plain map carries: a setup-binding component read
//! anchored at its tag, and the static member columns of rewritten reads.
//! The generator is instantiated once per mode, so the plain instantiation
//! is exactly the generator without those links and a compile without a map
//! runs none of their work.

/// A render map a generator records.
pub trait RenderMapMode: 'static {
    /// The map the SFC assembly composes into the component's.
    const COMPOSED: bool;
}

/// The map a standalone compile returns, and what every compile records.
pub enum PlainRenderMap {}

impl RenderMapMode for PlainRenderMap {
    const COMPOSED: bool = false;
}

/// The map the SFC assembly composes into the component's map.
pub enum ComposedRenderMap {}

impl RenderMapMode for ComposedRenderMap {
    const COMPOSED: bool = true;
}
