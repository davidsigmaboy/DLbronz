//! DLbronz web core: HTML5 parsing (html5ever), CSS cascade, block/inline layout.
//!
//! `doc`, `css`, `layout` and `render` have no dependencies, so they can be unit
//! tested with plain `rustc --test --cfg dlbronz_core_only` (see scripts/test-core.sh).

pub mod css;
pub mod doc;
pub mod layout;
pub mod render;

#[cfg(not(dlbronz_core_only))]
pub mod ffi;
#[cfg(not(dlbronz_core_only))]
pub mod parse;

#[cfg(test)]
mod tests;
