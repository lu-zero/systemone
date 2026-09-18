use crate::questions::choice;
use crate::types::{EntryType, Question};

/// Implemented by enums that describe their own [`choice`] criteria instead of listing the
/// labels by hand. Implement it directly, derive it with `systemone-macro`, or wire it up
/// with `systemone-facet`'s `impl_choice_criteria!`.
pub trait ChoiceCriteria {
    /// Labels mapped to their descriptions, in declaration order. `None` leaves a label
    /// undescribed.
    fn choice_criteria() -> Vec<(&'static str, Option<EntryType>)>;
}

/// Create a [`choice`] question from a type implementing [`ChoiceCriteria`].
pub fn choice_typed<T: ChoiceCriteria>(instructions: EntryType) -> Question {
    choice(instructions, T::choice_criteria())
}
