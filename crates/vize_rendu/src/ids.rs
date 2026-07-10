//! Stable typed indices into a Rendu root's owned arenas.

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            pub const fn index(self) -> usize {
                self.0 as usize
            }

            pub const fn raw(self) -> u32 {
                self.0
            }

            pub(crate) fn from_index(index: usize) -> Self {
                Self(u32::try_from(index).expect("Rendu arena exceeds u32::MAX entries"))
            }
        }
    };
}

define_id!(RenduSourceId);
define_id!(RenduExpressionId);
define_id!(RenduNodeId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_their_arena_index() {
        let id = RenduNodeId::from_index(42);
        assert_eq!(id.index(), 42);
        assert_eq!(id.raw(), 42);
    }
}
