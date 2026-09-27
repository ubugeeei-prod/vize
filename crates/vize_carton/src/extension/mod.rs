//! External wire boundaries owned by L0.
//!
//! First-party native producers must use typed level interfaces rather than
//! serializing these pages between levels. Guest runtime packaging remains
//! unfinished until the real L0 extraction can isolate its no_std feature graph.

pub mod handshake;
pub mod wire;
