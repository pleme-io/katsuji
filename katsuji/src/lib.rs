//! # Katsuji (活字) — movable type for the terminal
//!
//! *Katsuji* is the movable type of the printing press: the physical
//! pieces a line is **composed from**. That is the whole idea here. A
//! terminal line is set from typed pieces — an ink, an attribute, a
//! glyph that is known to render — and never concatenated from escape
//! strings.
//!
//! ```
//! use katsuji::{Attr, Crisp, Ink, Line, Piece};
//!
//! let rule = Line::new()
//!     .piece(Piece::glyphs(Crisp::Horizontal, 8).ink(Ink::Cyan))
//!     .gap(2)
//!     .piece(Piece::text("mado").attr(Attr::Bold))
//!     .gap(1)
//!     .piece(Piece::text("0.1.98").attr(Attr::Dim));
//!
//! assert_eq!(rule.plain(), "────────  mado 0.1.98");
//! assert!(rule.render().contains('\u{1b}'));
//! ```
//!
//! ## Why this exists
//!
//! ★★ TYPED EMISSION says every emitted string comes from a typed
//! surface and `format!()` of syntax is banned. For Nix there is a typed
//! AST; for Go and YAML there are emitters. For the TERMINAL there was
//! nothing — so ANSI stayed the one place in the fleet where a `\x1b[36m`
//! in a `format!()` was normal. Katsuji is the missing surface.
//!
//! It is deliberately a **portable library, not a mado module**: the same
//! system serves mado, banken, tear, izumi, frost and any CLI that prints
//! in colour, so the textscape is colocated once and redistributed rather
//! than re-derived per consumer.
//!
//! ## The five illegal states, and how each is removed
//!
//! Gate 0 was written before any technique was chosen — you cannot pick
//! the tool that removes a bad value until you have named it.
//!
//! | # | illegal state | technique | tier |
//! |---|---|---|---|
//! | 1 | a glyph with no font geometry (tofu) | [`Crisp`] has no variant for `╭` — the 139 no-geometry chars are unnameable | truly-unrepresentable |
//! | 2 | a hardcoded hex colour | [`Ink`] is a fieldless enum of semantic slots; no `from_hex`, no `Rgb` | truly-unrepresentable |
//! | 3 | unbalanced SGR (bold bleeds forever) | style travels WITH content; open and close are emitted by one line of code | truly-unrepresentable |
//! | 4 | a raw `\x1b` from a consumer | no escape is spelled here: bytes come from kazari's `StyleAtom`, in one private fn; consumers never see one | parse-time-rejected |
//! | 5 | a wrong width claim | [`Line::width`] is COMPUTED, never asserted | eval-caught — see below |
//!
//! ### Tier honesty
//!
//! State 5 is **eval-caught, not unrepresentable**, and that is stated
//! rather than rounded up. Width is computed from `char` counts, which is
//! correct for crisp glyphs (single-width by construction) and WRONG for
//! CJK and emoji (double-width) and combining marks (zero-width). A real
//! `unicode-width` seam is M1. Until then, alignment of such text is the
//! caller's problem and [`Line::width`] is a lower bound.
//!
//! Also honest: the `(deftexto …)` tatara-lisp authoring form is a named
//! **M1**. `specs/textscape.lisp` documents the destination surface; M0
//! is typed Rust. A documented form is not a wired form.
//!
//! ## What it is not
//!
//! Not a layout engine, not a TUI framework, not a colour system. It
//! composes ONE line from typed pieces. Colour VALUES belong to
//! `ishou`/`irodori` — katsuji emits the semantic slot and lets the theme
//! resolve it, which is exactly why an ishou retune needs no change here.
//! Multi-line arrangement stays with the consumer.
//!
//! ## The typographic position
//!
//! Crispness is contrast plus the absence of fuzz. The dither glyphs
//! `░▒▓` are 25/50/75% checkerboards and read as noise at cell size, so
//! [`Crisp::is_dither`] surfaces that as a queryable fact rather than
//! leaving it as folklore. Restraint is the house style: one accent
//! slot, dim for de-emphasis, whitespace doing the work — which is why
//! [`Line::gap`] is a first-class verb and not a `text("   ")`.

#![forbid(unsafe_code)]

pub mod compose;
pub mod glyph;
pub mod ink;
pub mod sgr;

pub use compose::{Line, Piece};
pub use glyph::Crisp;
pub use ink::Ink;
pub use sgr::Attr;
pub use kazari::{Capability, ColorLevel, Role, Stream};

#[cfg(test)]
mod contract {
    use super::*;

    /// The banner mado ships, composed from typed pieces — the first
    /// consumer, and the proof the surface is expressive enough for real
    /// decorative work.
    fn window_banner(version: &str) -> Vec<Line> {
        let c = Ink::Cyan;
        let edge = |g: Crisp, n: usize| Piece::glyphs(g, n).ink(c);
        vec![
            Line::new()
                .piece(edge(Crisp::CornerTopLeft, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::TeeDown, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::CornerTopRight, 1)),
            Line::new()
                .piece(edge(Crisp::Vertical, 1))
                .gap(3)
                .piece(edge(Crisp::Vertical, 1))
                .gap(3)
                .piece(edge(Crisp::Vertical, 1))
                .gap(4)
                .piece(Piece::text("mado").attr(Attr::Bold))
                .gap(2)
                .piece(Piece::text(version).attr(Attr::Dim)),
            Line::new()
                .piece(edge(Crisp::TeeLeft, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::Cross, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::TeeRight, 1)),
            Line::new()
                .piece(edge(Crisp::Vertical, 1))
                .gap(3)
                .piece(edge(Crisp::Vertical, 1))
                .gap(3)
                .piece(edge(Crisp::Vertical, 1))
                .gap(4)
                .piece(Piece::text("gpu terminal").attr(Attr::Dim)),
            Line::new()
                .piece(edge(Crisp::CornerBottomLeft, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::TeeUp, 1))
                .piece(edge(Crisp::Horizontal, 3))
                .piece(edge(Crisp::CornerBottomRight, 1)),
        ]
    }

    #[test]
    fn the_banner_frame_is_square() {
        let b = window_banner("0.1.98");
        let frame: Vec<usize> = b.iter().map(|l| l.plain().chars().take(9).count()).collect();
        assert!(frame.iter().all(|&w| w == 9), "frame rows disagree: {frame:?}");
    }

    #[test]
    fn the_banner_uses_only_glyphs_that_render() {
        for line in window_banner("0.1.98") {
            for ch in line.plain().chars() {
                let decorative = !ch.is_ascii();
                if decorative {
                    assert!(
                        Crisp::ALL.iter().any(|g| g.ch() == ch),
                        "{ch:?} is not in the crisp set — it may render as tofu"
                    );
                }
            }
        }
    }

    #[test]
    fn the_banner_never_leaves_a_style_open() {
        for line in window_banner("0.1.98") {
            let s = line.render();
            if s.contains('\u{1b}') {
                assert!(s.ends_with("\u{1b}[0m"), "style left open: {s:?}");
            }
        }
    }

    #[test]
    fn the_banner_carries_the_version_it_was_given() {
        let s: String = window_banner("9.9.9").iter().map(Line::plain).collect();
        assert!(s.contains("9.9.9"), "version not rendered");
        assert!(!s.contains("0.1.0"), "a version was invented");
    }

    /// The plain projection is what a log or a pipe receives. It must be
    /// readable — the banner should degrade to something sensible, not
    /// to a wall of nothing.
    #[test]
    fn the_banner_degrades_to_clean_text() {
        let plain: Vec<String> = window_banner("0.1.98").iter().map(Line::plain).collect();
        assert!(plain.iter().all(|l| !l.contains('\u{1b}')));
        assert!(plain[1].contains("mado"));
        assert!(plain[1].contains("0.1.98"));
    }
}
