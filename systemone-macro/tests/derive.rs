use systemone::ChoiceCriteria;
use systemone_macro::ChoiceCriteria;

#[derive(ChoiceCriteria)]
#[allow(dead_code)]
enum Category {
    Billing,
    /// Technical issue.
    Technical,
}

#[test]
fn generates_lowercase_labels_and_doc_descriptions() {
    let criteria = Category::choice_criteria();
    assert_eq!(criteria[0], ("billing", None));
    assert_eq!(
        criteria[1],
        (
            "technical",
            Some(systemone::EntryType::from("Technical issue."))
        )
    );
}

#[derive(ChoiceCriteria)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
enum ShippingStatus {
    InTransit,
    #[serde(rename = "done")]
    Delivered,
}

#[test]
fn honors_serde_rename_attributes() {
    let criteria = ShippingStatus::choice_criteria();
    assert_eq!(criteria[0], ("in-transit", None));
    assert_eq!(criteria[1], ("done", None));
}
