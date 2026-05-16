mod heading_levels;
mod landmark_roles;
mod placeholder_label_option;
mod use_list;

use crate::rule::RuleRegistry;

pub use heading_levels::HeadingLevels;
pub use landmark_roles::LandmarkRoles;
pub use placeholder_label_option::PlaceholderLabelOption;
pub use use_list::UseList;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(HeadingLevels));
    registry.register(Box::new(LandmarkRoles));
    registry.register(Box::new(PlaceholderLabelOption));
    registry.register(Box::new(UseList));
}
