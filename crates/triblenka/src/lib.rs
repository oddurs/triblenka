//! Triblenka is a server-first web framework for content-driven sites.
//!
//! The project is at the design stage: the architecture is settled and recorded in `DESIGN.md`,
//! the intended API is documented under `docs/`, and the work is tracked in `ROADMAP.md`. This
//! crate is the shell those pieces will be built into.
//!
//! The design rests on one split: `src/**` is code, compiled into a binary, and `content/**` is
//! data, loaded into a typed store. A content change is therefore a data change rather than a
//! rebuild, which is what makes a content deploy independent of a code deploy.

/// The stage the project is currently at.
///
/// Every public item in this crate is expected to change until [`Stage::Design`] is behind us.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Architecture settled, implementation not started.
    Design,
    /// M0: proving the content and template rebuild boundaries.
    Spike,
}

/// The stage this build of the crate represents.
pub const STAGE: Stage = Stage::Design;

/// The version of this crate, taken from Cargo at compile time.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_its_stage_and_version() {
        assert_eq!(STAGE, Stage::Design);
        assert!(!version().is_empty());
    }
}
