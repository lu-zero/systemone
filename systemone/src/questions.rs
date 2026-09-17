use std::collections::BTreeMap;

use crate::error::Error;
use crate::types::{EntryType, NoulCriteria, Question};

/// Create a yes/no question. Pass [`NoulCriteria`] to describe the yes and no outcomes.
pub fn noul(instructions: EntryType, criteria: Option<NoulCriteria>) -> Question {
    Question::Noul {
        instructions: (!instructions.is_null()).then_some(instructions),
        criteria,
    }
}

/// Create a score question using an ordered rubric.
///
/// # Errors
/// Returns [`Error::TooFewScoreCriteria`] if `criteria` has fewer than two entries.
pub fn score(instructions: EntryType, criteria: Vec<EntryType>) -> Result<Question, Error> {
    if criteria.len() < 2 {
        return Err(Error::TooFewScoreCriteria {
            count: criteria.len(),
        });
    }
    Ok(Question::Score {
        instructions,
        criteria,
    })
}

/// Create a question that selects between named alternatives.
///
/// `None` leaves a label undescribed.
pub fn choice<L: Into<String>>(
    instructions: EntryType,
    criteria: impl IntoIterator<Item = (L, Option<EntryType>)>,
) -> Question {
    let criteria = criteria
        .into_iter()
        .map(|(label, description)| (label.into(), description.unwrap_or(EntryType::Null)))
        .collect::<BTreeMap<_, _>>();
    Question::Choice {
        instructions,
        criteria,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_rejects_fewer_than_two_criteria() {
        let err = score(EntryType::Null, vec![EntryType::from("only one")]).unwrap_err();
        assert!(matches!(err, Error::TooFewScoreCriteria { count: 1 }));
    }

    #[test]
    fn noul_drops_null_instructions() {
        let question = noul(EntryType::Null, None);
        assert!(matches!(
            question,
            Question::Noul {
                instructions: None,
                ..
            }
        ));
    }

    #[test]
    fn choice_collects_labels_into_a_map() {
        let question = choice(
            EntryType::from("category?"),
            [
                ("billing", None),
                ("technical", Some(EntryType::from("tech issues"))),
            ],
        );
        let Question::Choice { criteria, .. } = question else {
            panic!("expected a choice question");
        };
        assert_eq!(criteria.len(), 2);
        assert_eq!(criteria["technical"], EntryType::from("tech issues"));
        assert_eq!(criteria["billing"], EntryType::Null);
    }
}
