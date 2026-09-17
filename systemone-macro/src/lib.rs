//! Stub.
//!
//! Planned: a derive macro that turns a plain enum into both the outbound `choice`/`score`
//! criteria (from its variants) and the typed response value deserialized by
//! [`systemone::ChoiceResponse`](../systemone/struct.ChoiceResponse.html), so the enum is the
//! single source of truth instead of hand-writing both sides.
//!
//! See `systemone-facet` for an alternative approach based on reflection instead of a
//! proc-macro; neither is implemented yet.
