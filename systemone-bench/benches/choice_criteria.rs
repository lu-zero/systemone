//! Compares `systemone-macro`'s compile-time codegen against `systemone-facet`'s runtime
//! reflection for building the same `choice` criteria.
//!
//! Both measure steady state only: `systemone-facet`'s lowercase-label cache is filled once
//! per process, and that cost is not visible here.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use systemone::ChoiceCriteria;

#[derive(systemone_macro::ChoiceCriteria)]
#[allow(dead_code)]
enum CategoryMacro {
    /// Billing question or dispute.
    Billing,
    /// Technical issue with the product.
    Technical,
    Other,
}

#[derive(facet::Facet)]
#[repr(u8)]
enum CategoryFacet {
    /// Billing question or dispute.
    Billing,
    /// Technical issue with the product.
    Technical,
    Other,
}
systemone_facet::impl_choice_criteria!(CategoryFacet);

fn bench_choice_criteria(c: &mut Criterion) {
    // Warm the facet cache once, outside the measured loop.
    let _ = CategoryFacet::choice_criteria();

    let mut group = c.benchmark_group("choice_criteria");
    group.bench_function("macro", |b| {
        b.iter(|| black_box(CategoryMacro::choice_criteria()))
    });
    group.bench_function("facet", |b| {
        b.iter(|| black_box(CategoryFacet::choice_criteria()))
    });
    group.finish();
}

criterion_group!(benches, bench_choice_criteria);
criterion_main!(benches);
