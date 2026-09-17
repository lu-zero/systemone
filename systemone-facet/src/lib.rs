//! Typed `choice` criteria via [`facet`] reflection, as an alternative to
//! `systemone-macro`'s derive-macro approach.
//!
//! Derive `facet::Facet` (not this crate's own macro — there isn't one) on a unit-variant
//! enum, then wire it to `systemone::ChoiceCriteria` with [`impl_choice_criteria!`]:
//!
//! ```
//! use facet::Facet;
//!
//! #[derive(Facet)]
//! #[repr(u8)]
//! enum Category {
//!     /// Billing question or dispute.
//!     Billing,
//!     Technical,
//! }
//! systemone_facet::impl_choice_criteria!(Category);
//! ```
//!
//! Labels use `#[facet(rename = "...")]` / `#[facet(rename_all = "...")]` when present,
//! else the variant name lowercased, matching `systemone-macro`'s default so the two are
//! directly comparable for a plain enum. Descriptions come from each variant's doc comment.
//! Unlike the macro, which bakes labels in at compile time, a lowercased fallback here is
//! computed (and cached) the first time each variant name is seen.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use facet::{Facet, Type, UserType};
use systemone::EntryType;

/// Build `choice` criteria for `T` via runtime reflection over its `facet::Facet` shape.
///
/// # Panics
/// Panics if `T::SHAPE` is not a user-defined enum.
pub fn choice_criteria<'a, T: Facet<'a>>() -> Vec<(&'static str, Option<EntryType>)> {
    let Type::User(UserType::Enum(enum_type)) = &T::SHAPE.ty else {
        panic!("systemone_facet::choice_criteria::<T> requires T to be an enum");
    };
    enum_type
        .variants
        .iter()
        .map(|variant| {
            let label = variant.rename.unwrap_or_else(|| lowercased(variant.name));
            let description = (!variant.doc.is_empty()).then(|| {
                EntryType::from(
                    variant
                        .doc
                        .iter()
                        .map(|line| line.trim())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            });
            (label, description)
        })
        .collect()
}

/// Lowercase `name`, caching the leaked `'static` result so repeated calls for the same
/// variant name don't each leak a new allocation.
fn lowercased(name: &'static str) -> &'static str {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, &'static str>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    cache
        .entry(name)
        .or_insert_with(|| Box::leak(name.to_lowercase().into_boxed_str()))
}

/// Implement [`systemone::ChoiceCriteria`] for `$ty` by delegating to [`choice_criteria`].
#[macro_export]
macro_rules! impl_choice_criteria {
    ($ty:ty) => {
        impl ::systemone::ChoiceCriteria for $ty {
            fn choice_criteria()
            -> ::std::vec::Vec<(&'static str, ::std::option::Option<::systemone::EntryType>)> {
                $crate::choice_criteria::<$ty>()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use systemone::ChoiceCriteria;

    use super::*;

    #[derive(Facet)]
    #[repr(u8)]
    enum Category {
        /// Billing question or dispute.
        Billing,
        Technical,
    }
    impl_choice_criteria!(Category);

    #[test]
    fn labels_default_to_lowercase_variant_names() {
        let criteria = Category::choice_criteria();
        assert_eq!(criteria[0].0, "billing");
        assert_eq!(criteria[1].0, "technical");
    }

    #[test]
    fn descriptions_come_from_doc_comments() {
        let criteria = Category::choice_criteria();
        assert_eq!(
            criteria[0].1,
            Some(EntryType::from("Billing question or dispute."))
        );
        assert_eq!(criteria[1].1, None);
    }

    #[test]
    #[should_panic(expected = "requires T to be an enum")]
    fn panics_for_non_enum_shapes() {
        #[derive(Facet)]
        struct NotAnEnum {
            #[allow(dead_code)]
            field: u8,
        }
        let _ = choice_criteria::<NotAnEnum>();
    }
}
