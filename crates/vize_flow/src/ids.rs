use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[doc = concat!("Opaque identifier for a `", stringify!($name), "` graph entity.")]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            /// Reconstruct an ID at an interchange boundary.
            ///
            /// Graph mutation APIs still validate that the ID belongs to the
            /// graph receiving it.
            #[inline]
            pub const fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            /// Return the stable integer representation of this ID.
            #[inline]
            pub const fn raw(self) -> u32 {
                self.0
            }

            #[inline]
            pub(crate) const fn index(self) -> usize {
                self.0 as usize
            }

            #[inline]
            pub(crate) const fn from_index(index: usize) -> Self {
                Self(index as u32)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_id!(SourceId);
define_id!(BlockId);
define_id!(NodeId);
define_id!(ValueId);
define_id!(SymbolId);
define_id!(ControlEdgeId);
define_id!(DataEdgeId);
define_id!(EffectId);
define_id!(EffectEdgeId);
