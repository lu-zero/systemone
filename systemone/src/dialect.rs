use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::error::Error;
use crate::types::{NoulCriteria, Question, Questions, SystemOneRequest};

pub(crate) const DECISIONS_PATH: &str = "/v1/decisions";

const MIN_OPTIONS: usize = 2;
const MAX_SCORE_LEVELS: usize = 10;
const TRUE_WHEN: &str = "True when: ";
const FALSE_WHEN: &str = "False when: ";
const USER_ROLE: &str = "user";
const INPUT_JSON: &str = "input_json";
const PREDICATE: &str = "predicate";
const CHOICE: &str = "choice";
const SCORE: &str = "score";
const REFUSAL: &str = "refusal";

/// Which decision API a [`Client`](crate::Client) speaks.
///
/// [`Dialect::SystemOne`] posts the request as written to `/v1/systemone`.
/// [`Dialect::OpenAi`] posts the same questions to `/v1/decisions`. Host, key,
/// and the default model stay as configured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Dialect {
    /// TypeSafe `POST /v1/systemone`.
    #[default]
    SystemOne,
    /// OpenAI Decisions `POST /v1/decisions`.
    OpenAi,
}

pub(crate) fn encode(request: &SystemOneRequest) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(&encode_value(request)?)?)
}

fn encode_value(request: &SystemOneRequest) -> Result<Value, Error> {
    let model = request
        .model
        .as_deref()
        .filter(|model| !model.is_empty())
        .ok_or(Error::MissingModel)?;
    let mut questions = Vec::with_capacity(request.questions.len());
    for (name, question) in &request.questions {
        questions.push(encode_question(name, question)?);
    }
    Ok(json!({
        "model": model,
        "input": encode_input(&request.state),
        "questions": questions,
    }))
}

fn encode_input(state: &Value) -> Value {
    match state {
        Value::String(text) => Value::String(text.clone()),
        Value::Array(_) | Value::Object(_) => json!([{
            "role": USER_ROLE,
            "content": [{ "type": INPUT_JSON, "data": state }],
        }]),
        other => Value::String(other.to_string()),
    }
}

fn encode_question(name: &str, question: &Question) -> Result<Value, Error> {
    match question {
        Question::Noul {
            instructions,
            criteria,
        } => Ok(json!({
            "type": PREDICATE,
            "name": name,
            "instructions": predicate_instructions(name, instructions.as_ref(), criteria.as_ref())?,
        })),
        Question::Choice {
            instructions,
            criteria,
        } => {
            if criteria.len() < MIN_OPTIONS {
                return Err(Error::TooFewChoices {
                    question: name.to_string(),
                    count: criteria.len(),
                });
            }
            let mut choices = Vec::with_capacity(criteria.len());
            for (label, description) in criteria {
                choices.push(match rendered(description) {
                    Some(description) => json!({"value": label, "description": description}),
                    None => json!({"value": label}),
                });
            }
            Ok(json!({
                "type": CHOICE,
                "name": name,
                "instructions": required_instructions(name, instructions)?,
                "choices": choices,
            }))
        }
        Question::Score {
            instructions,
            criteria,
        } => {
            if criteria.len() < MIN_OPTIONS {
                return Err(Error::TooFewScoreCriteria {
                    count: criteria.len(),
                });
            }
            if criteria.len() > MAX_SCORE_LEVELS {
                return Err(Error::TooManyScoreLevels {
                    question: name.to_string(),
                    count: criteria.len(),
                });
            }
            let levels: Vec<_> = criteria
                .iter()
                .enumerate()
                .map(|(index, level)| json!({"label": rendered(level).unwrap_or_else(|| index.to_string())}))
                .collect();
            Ok(json!({
                "type": SCORE,
                "name": name,
                "instructions": required_instructions(name, instructions)?,
                "levels": levels,
            }))
        }
    }
}

fn predicate_instructions(
    name: &str,
    instructions: Option<&Value>,
    criteria: Option<&NoulCriteria>,
) -> Result<String, Error> {
    let mut lines = Vec::new();
    if let Some(text) = instructions.and_then(rendered) {
        lines.push(text);
    }
    if let Some(criteria) = criteria {
        if let Some(text) = criteria.is_true.as_ref().and_then(rendered) {
            lines.push(format!("{TRUE_WHEN}{text}"));
        }
        if let Some(text) = criteria.is_false.as_ref().and_then(rendered) {
            lines.push(format!("{FALSE_WHEN}{text}"));
        }
    }
    if lines.is_empty() {
        return Err(Error::MissingInstructions {
            question: name.to_string(),
        });
    }
    Ok(lines.join("\n"))
}

fn required_instructions(name: &str, instructions: &Value) -> Result<String, Error> {
    rendered(instructions).ok_or_else(|| Error::MissingInstructions {
        question: name.to_string(),
    })
}

fn rendered(value: &Value) -> Option<String> {
    let text = match value {
        Value::Null => return None,
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    (!text.is_empty()).then_some(text)
}

pub(crate) fn decode(body: &str, questions: &Questions) -> Result<Value, Error> {
    let response: DecisionResponse = serde_json::from_str(body)?;
    let mut answers = Map::new();
    for answer in response.answers {
        let Some(question) = questions.get(&answer.name) else {
            continue;
        };
        if answer.kind == REFUSAL {
            return Err(Error::Refused {
                question: answer.name,
            });
        }
        if !kind_matches(question, &answer.kind) {
            return Err(Error::UnexpectedAnswer {
                question: answer.name,
                kind: answer.kind,
            });
        }
        let value = answer_value(question, &answer)?;
        answers.insert(answer.name, value);
    }
    for name in questions.keys() {
        if !answers.contains_key(name) {
            return Err(Error::MissingAnswer {
                question: name.clone(),
            });
        }
    }
    Ok(json!({
        "model": response.model,
        "answers": answers,
        "usage": {
            "input_tokens": response.usage.input_tokens,
            "output_tokens": response.usage.output_tokens,
        },
    }))
}

fn kind_matches(question: &Question, kind: &str) -> bool {
    match question {
        Question::Noul { .. } => kind == PREDICATE,
        Question::Choice { .. } => kind == CHOICE,
        Question::Score { .. } => kind == SCORE,
    }
}

fn answer_value(question: &Question, answer: &DecisionAnswer) -> Result<Value, Error> {
    match question {
        Question::Noul { .. } => {
            let noul = answer.probability.ok_or_else(|| Error::UnexpectedAnswer {
                question: answer.name.clone(),
                kind: answer.kind.clone(),
            })?;
            Ok(json!({"type": "noul", "noul": noul}))
        }
        Question::Choice { .. } => {
            let choice = answer
                .choice
                .as_ref()
                .ok_or_else(|| Error::UnexpectedAnswer {
                    question: answer.name.clone(),
                    kind: answer.kind.clone(),
                })?;
            Ok(json!({
                "type": "choice",
                "choice": json_key(choice),
                "confidence": answer.confidence.unwrap_or(0.0),
                "probabilities": probability_map(answer),
            }))
        }
        Question::Score { criteria, .. } => Ok(json!({
            "type": "score",
            "score": answer.score.ok_or_else(|| Error::UnexpectedAnswer {
                question: answer.name.clone(),
                kind: answer.kind.clone(),
            })?,
            "confidence": answer.confidence.unwrap_or(0.0),
            "legend": legend(criteria),
            "probabilities": probability_map(answer),
        })),
    }
}

fn probability_map(answer: &DecisionAnswer) -> Value {
    let mut probabilities = Map::new();
    for entry in answer.probabilities.iter().flatten() {
        probabilities.insert(json_key(&entry.value), json!(entry.probability));
    }
    Value::Object(probabilities)
}

fn legend(criteria: &[Value]) -> Value {
    let mut legend = Map::new();
    for (index, level) in criteria.iter().enumerate() {
        legend.insert(index.to_string(), level.clone());
    }
    Value::Object(legend)
}

fn json_key(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number
            .as_i64()
            .map(|index| index.to_string())
            .unwrap_or_else(|| number.to_string()),
        Value::Bool(value) => value.to_string(),
        other => other.to_string(),
    }
}

#[derive(Deserialize)]
struct DecisionResponse {
    model: String,
    answers: Vec<DecisionAnswer>,
    #[serde(default)]
    usage: DecisionUsage,
}

#[derive(Deserialize, Default)]
struct DecisionUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

#[derive(Deserialize)]
struct DecisionAnswer {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    probability: Option<f64>,
    choice: Option<Value>,
    score: Option<f64>,
    #[serde(default)]
    confidence: Option<f64>,
    probabilities: Option<Vec<DecisionProbability>>,
}

#[derive(Deserialize)]
struct DecisionProbability {
    value: Value,
    probability: f64,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::questions::{choice, noul, score};
    use crate::types::EntryType;

    fn request(questions: Questions) -> SystemOneRequest {
        SystemOneRequest {
            state: EntryType::from("charged twice"),
            questions,
            model: Some("gpt-6-luna".to_string()),
        }
    }

    #[test]
    fn encodes_string_state_and_all_three_question_types() {
        let mut questions = BTreeMap::new();
        questions.insert(
            "refund".to_string(),
            noul(
                "Does the customer want a refund?".into(),
                Some(NoulCriteria {
                    is_true: Some(EntryType::from("asks for money back")),
                    is_false: None,
                }),
            ),
        );
        questions.insert(
            "team".to_string(),
            choice(
                "Which team?".into(),
                [
                    ("billing", Some(EntryType::from("payments"))),
                    ("other", None),
                ],
            ),
        );
        questions.insert(
            "urgency".to_string(),
            score(
                "How urgent?".into(),
                vec![EntryType::from("low"), EntryType::from("high")],
            )
            .unwrap(),
        );

        let body = encode_value(&request(questions)).unwrap();
        assert_eq!(
            body,
            json!({
                "model": "gpt-6-luna",
                "input": "charged twice",
                "questions": [
                    {
                        "type": "predicate",
                        "name": "refund",
                        "instructions": "Does the customer want a refund?\nTrue when: asks for money back",
                    },
                    {
                        "type": "choice",
                        "name": "team",
                        "instructions": "Which team?",
                        "choices": [
                            {"value": "billing", "description": "payments"},
                            {"value": "other"},
                        ],
                    },
                    {
                        "type": "score",
                        "name": "urgency",
                        "instructions": "How urgent?",
                        "levels": [{"label": "low"}, {"label": "high"}],
                    },
                ],
            })
        );
    }

    #[test]
    fn encodes_object_state_as_input_json() {
        let mut questions = BTreeMap::new();
        questions.insert("refund".to_string(), noul("Refund?".into(), None));
        let mut request = request(questions);
        request.state = json!({"ticket": "charged twice"});

        let body = encode_value(&request).unwrap();
        assert_eq!(
            body["input"],
            json!([{
                "role": "user",
                "content": [{
                    "type": "input_json",
                    "data": {"ticket": "charged twice"},
                }],
            }])
        );
    }

    #[test]
    fn rejects_a_question_openai_cannot_represent() {
        let bare = noul(EntryType::Null, None);
        let one_choice = choice("Which?".into(), [("only", None)]);
        let many_levels = score(
            "How?".into(),
            (0..11)
                .map(|index| EntryType::from(index.to_string()))
                .collect(),
        )
        .unwrap();

        for (name, question, expected) in [
            (
                "bare",
                bare,
                Error::MissingInstructions {
                    question: "bare".to_string(),
                },
            ),
            (
                "only",
                one_choice,
                Error::TooFewChoices {
                    question: "only".to_string(),
                    count: 1,
                },
            ),
            (
                "scale",
                many_levels,
                Error::TooManyScoreLevels {
                    question: "scale".to_string(),
                    count: 11,
                },
            ),
        ] {
            let mut questions = BTreeMap::new();
            questions.insert(name.to_string(), question);
            let err = encode_value(&request(questions)).unwrap_err();
            assert_eq!(err.to_string(), expected.to_string());
        }
    }

    #[test]
    fn decodes_answers_back_into_systemone_shape() {
        let mut questions = BTreeMap::new();
        questions.insert("refund".to_string(), noul("Refund?".into(), None));
        questions.insert(
            "urgency".to_string(),
            score(
                "How urgent?".into(),
                vec![EntryType::from("low"), EntryType::from("high")],
            )
            .unwrap(),
        );
        let body = r#"{
            "model": "gpt-6-luna",
            "answers": [
                {"type": "predicate", "name": "refund", "probability": 0.91},
                {
                    "type": "score",
                    "name": "urgency",
                    "score": 0.4,
                    "confidence": 0.5,
                    "probabilities": [
                        {"value": 0, "label": "low", "probability": 0.6},
                        {"value": 1, "label": "high", "probability": 0.4}
                    ]
                }
            ],
            "usage": {"input_tokens": 12}
        }"#;

        let decoded = decode(body, &questions).unwrap();
        assert_eq!(
            decoded["answers"]["refund"],
            json!({"type": "noul", "noul": 0.91})
        );
        assert_eq!(
            decoded["answers"]["urgency"],
            json!({
                "type": "score",
                "score": 0.4,
                "confidence": 0.5,
                "legend": {"0": "low", "1": "high"},
                "probabilities": {"0": 0.6, "1": 0.4},
            })
        );
        assert_eq!(decoded["usage"]["output_tokens"], json!(0));
    }

    #[test]
    fn refusal_and_a_missing_answer_are_errors() {
        let mut questions = BTreeMap::new();
        questions.insert("refund".to_string(), noul("Refund?".into(), None));

        let refused = decode(
            r#"{"model":"gpt-6-luna","answers":[{"type":"refusal","name":"refund"}]}"#,
            &questions,
        )
        .unwrap_err();
        assert!(matches!(refused, Error::Refused { question } if question == "refund"));

        let missing = decode(r#"{"model":"gpt-6-luna","answers":[]}"#, &questions).unwrap_err();
        assert!(matches!(missing, Error::MissingAnswer { question } if question == "refund"));
    }
}
