//! Static hoisting analysis for Vue templates.
//!
//! Identifies static content that can be hoisted out of the render function.

use vize_carton::{bitflags, CompactString, SmallVec};

bitflags! {
    /// Patch flags indicating what parts of a node can change
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PatchFlags: i32 {
        /// Dynamic text content
        const TEXT = 1;
        /// Dynamic class binding
        const CLASS = 1 << 1;
        /// Dynamic style binding
        const STYLE = 1 << 2;
        /// Dynamic non-class/style props
        const PROPS = 1 << 3;
        /// Props with dynamic keys
        const FULL_PROPS = 1 << 4;
        /// Element needs props hydration
        const NEED_HYDRATION = 1 << 5;
        /// Children order may change
        const STABLE_FRAGMENT = 1 << 6;
        /// Fragment with keyed children
        const KEYED_FRAGMENT = 1 << 7;
        /// Fragment with unkeyed children
        const UNKEYED_FRAGMENT = 1 << 8;
        /// Only non-props patch needed
        const NEED_PATCH = 1 << 9;
        /// Dynamic slots
        const DYNAMIC_SLOTS = 1 << 10;
        /// Dev only: HMR
        const DEV_ROOT_FRAGMENT = 1 << 11;
        /// Static node (hoistable)
        const HOISTED = -1i32;
        /// Bail out of optimization
        const BAIL = -2i32;
    }
}

impl PatchFlags {
    /// Get flag names for display
    pub fn flag_names(&self) -> SmallVec<[&'static str; 8]> {
        let mut names = SmallVec::new();
        if self.contains(Self::TEXT) {
            names.push("TEXT");
        }
        if self.contains(Self::CLASS) {
            names.push("CLASS");
        }
        if self.contains(Self::STYLE) {
            names.push("STYLE");
        }
        if self.contains(Self::PROPS) {
            names.push("PROPS");
        }
        if self.contains(Self::FULL_PROPS) {
            names.push("FULL_PROPS");
        }
        if self.contains(Self::NEED_HYDRATION) {
            names.push("NEED_HYDRATION");
        }
        if self.contains(Self::STABLE_FRAGMENT) {
            names.push("STABLE_FRAGMENT");
        }
        if self.contains(Self::KEYED_FRAGMENT) {
            names.push("KEYED_FRAGMENT");
        }
        if self.contains(Self::UNKEYED_FRAGMENT) {
            names.push("UNKEYED_FRAGMENT");
        }
        if self.contains(Self::NEED_PATCH) {
            names.push("NEED_PATCH");
        }
        if self.contains(Self::DYNAMIC_SLOTS) {
            names.push("DYNAMIC_SLOTS");
        }
        if *self == Self::HOISTED {
            names.push("HOISTED");
        }
        if *self == Self::BAIL {
            names.push("BAIL");
        }
        names
    }
}

/// Hoisting level for an expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum HoistLevel {
    /// Cannot be hoisted
    NotHoistable = 0,
    /// Can be hoisted as a string constant
    StringConstant = 1,
    /// Can be hoisted as a VNode constant
    Constant = 2,
    /// Can be hoisted as a VNode that may contain runtime helpers
    Full = 3,
}

/// Unique identifier for a hoisted expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HoistedId(u32);

impl HoistedId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// A hoisted expression
#[derive(Debug, Clone)]
pub struct HoistedExpr {
    pub id: HoistedId,
    pub level: HoistLevel,
    pub content: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Analysis result for a node
#[derive(Debug, Clone)]
pub struct NodeAnalysis {
    pub hoist_level: HoistLevel,
    pub patch_flags: PatchFlags,
    pub dynamic_props: SmallVec<[CompactString; 4]>,
    pub is_static: bool,
}

impl Default for NodeAnalysis {
    fn default() -> Self {
        Self {
            hoist_level: HoistLevel::NotHoistable,
            patch_flags: PatchFlags::empty(),
            dynamic_props: SmallVec::new(),
            is_static: false,
        }
    }
}

/// Statistics about hoisting
#[derive(Debug, Clone, Default)]
pub struct HoistStats {
    pub total_hoists: u32,
    pub string_constants: u32,
    pub vnode_constants: u32,
    pub full_hoists: u32,
}

/// Tracks hoisting during template compilation
#[derive(Debug, Default)]
pub struct HoistTracker {
    hoists: Vec<HoistedExpr>,
    next_id: u32,
}

impl HoistTracker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a hoisted expression
    pub fn add_hoist(
        &mut self,
        level: HoistLevel,
        content: CompactString,
        start: u32,
        end: u32,
    ) -> HoistedId {
        let id = HoistedId::new(self.next_id);
        self.next_id += 1;

        self.hoists.push(HoistedExpr {
            id,
            level,
            content,
            start,
            end,
        });

        id
    }

    /// Get all hoisted expressions
    #[inline]
    pub fn hoists(&self) -> &[HoistedExpr] {
        &self.hoists
    }

    /// Get hoist count
    #[inline]
    pub fn count(&self) -> usize {
        self.hoists.len()
    }

    /// Get hoisting statistics
    pub fn stats(&self) -> HoistStats {
        let mut stats = HoistStats {
            total_hoists: self.hoists.len() as u32,
            ..Default::default()
        };

        for hoist in &self.hoists {
            match hoist.level {
                HoistLevel::StringConstant => stats.string_constants += 1,
                HoistLevel::Constant => stats.vnode_constants += 1,
                HoistLevel::Full => stats.full_hoists += 1,
                HoistLevel::NotHoistable => {}
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::{HoistLevel, HoistTracker, PatchFlags};
    use vize_carton::CompactString;

    #[test]
    fn test_patch_flags() {
        let flags = PatchFlags::TEXT | PatchFlags::CLASS;
        assert!(flags.contains(PatchFlags::TEXT));
        assert!(flags.contains(PatchFlags::CLASS));
        assert!(!flags.contains(PatchFlags::STYLE));

        let names = flags.flag_names();
        assert!(names.contains(&"TEXT"));
        assert!(names.contains(&"CLASS"));
    }

    #[test]
    fn test_hoist_tracker() {
        let mut tracker = HoistTracker::new();

        let id = tracker.add_hoist(
            HoistLevel::Constant,
            CompactString::new("_hoisted_1"),
            0,
            10,
        );

        assert_eq!(id.as_u32(), 0);
        assert_eq!(tracker.count(), 1);

        let stats = tracker.stats();
        assert_eq!(stats.total_hoists, 1);
        assert_eq!(stats.vnode_constants, 1);
    }
}
