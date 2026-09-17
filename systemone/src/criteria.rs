use crate::questions::choice;
use crate::types::{EntryType, Question};

/// Implemented by enums that describe their own [`choice`] criteria — the labels and,
/// optionally, their descriptions sent to the API — instead of listing them by hand.
///
/// Implement this yourself, derive it via `systemone-macro`'s `#[derive(ChoiceCriteria)]`
/// (compile-time codegen from an enum's variants and doc comments), or wire it up via
/// `systemone-facet`'s `impl_choice_criteria!` (runtime reflection over a `facet::Facet`
/// type). See those crates for the tradeoffs between the two.
pub trait ChoiceCriteria {
    /// Labels mapped to their descriptions, in declaration order. `None` leaves a label
    /// undescribed.
    fn choice_criteria() -> Vec<(&'static str, Option<EntryType>)>;
}

/// Create a [`choice`] question from a type implementing [`ChoiceCriteria`], instead of
/// listing labels by hand.
pub fn choice_typed<T: ChoiceCriteria>(instructions: EntryType) -> Question {
    choice(instructions, T::choice_criteria())
}
