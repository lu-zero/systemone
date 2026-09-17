//! Compares `systemone-macro`'s compile-time codegen against `systemone-facet`'s runtime
//! reflection for building the same `choice` criteria from an equivalent enum.
//!
//! Both benches measure steady state: `systemone-facet`'s lowercase-label cache (see its
//! `lowercased` function) is populated by the first call for a given variant name and
//! reused after that, so a real cold-start cost (one hashmap insert + a leaked allocation
//! per variant, once per process) isn't visible here.

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
