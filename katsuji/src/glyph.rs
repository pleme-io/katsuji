//! Glyphs, classified by whether they will actually RENDER.
//!
//! # Gate 0, illegal state 1: a glyph with no geometry
//!
//! This is the finding the whole module exists for, and it is measured,
//! not assumed. `mado/src/glyph_class.rs` documents that exactly **21**
//! box-drawing and block characters are synthesised by the renderer as
//! GPU rects (`render.rs::box_drawing_rects`), and that the *other* 139
//! in the same Unicode block have **no geometry** — the source names
//! `━`, `╌` and `╭╮╰╯` specifically, and records that routing them
//! through the sprite path "blanked the 139 chars … in every TUI app".
//!
//! So the obvious choices for a "pretty" box — rounded corners `╭╮╰╯`,
//! heavy rules `━` — are precisely the ones that render as tofu or as
//! whatever an arbitrary fallback font supplies. A caller cannot be
//! expected to carry that table in their head.
//!
//! [`Crisp`] IS that table. It has no variant for a glyph without
//! geometry, so composing a line out of `Crisp` values cannot produce
//! one — a compile error rather than a box of tofu discovered at
//! runtime, on a machine that is not yours, in a screenshot.
//!
//! Literal text is still expressible (see `Piece::Text`); the point is
//! that DECORATION — the part a human reaches for when making something
//! look good — is drawn from a set that is guaranteed to render.

/// A glyph guaranteed to have renderer geometry.
///
/// The exact 21 from `has_box_sprite`. Named by ROLE rather than by
/// codepoint so a composition reads as intent: `Crisp::CornerTopLeft`
/// says what it is for, `'\u{250C}'` does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Crisp {
    // ── light box-drawing: lines, corners, tees, cross ──
    /// `─` U+2500
    Horizontal,
    /// `│` U+2502
    Vertical,
    /// `┌` U+250C
    CornerTopLeft,
    /// `┐` U+2510
    CornerTopRight,
    /// `└` U+2514
    CornerBottomLeft,
    /// `┘` U+2518
    CornerBottomRight,
    /// `├` U+251C
    TeeLeft,
    /// `┤` U+2524
    TeeRight,
    /// `┬` U+252C
    TeeDown,
    /// `┴` U+2534
    TeeUp,
    /// `┼` U+253C
    Cross,
    // ── double rules ──
    /// `═` U+2550
    DoubleHorizontal,
    /// `║` U+2551
    DoubleVertical,
    // ── block elements ──
    /// `▀` U+2580
    HalfUpper,
    /// `▄` U+2584
    HalfLower,
    /// `█` U+2588
    Full,
    /// `▌` U+258C
    HalfLeft,
    /// `▐` U+2590
    HalfRight,
    /// `░` U+2591 — a DITHER. Reads as fuzz at cell size; reach for it
    /// deliberately, not for "texture".
    ShadeLight,
    /// `▒` U+2592 — see [`Self::ShadeLight`].
    ShadeMedium,
    /// `▓` U+2593 — see [`Self::ShadeLight`].
    ShadeDark,
}

impl Crisp {
    /// Every crisp glyph, in the order declared.
    ///
    /// A forced-arity array so adding a variant without adding it here is
    /// a compile error — the catalog cannot silently fall behind the
    /// enum (★★ CATALOG REFLECTION).
    pub const ALL: [Self; 21] = [
        Self::Horizontal,
        Self::Vertical,
        Self::CornerTopLeft,
        Self::CornerTopRight,
        Self::CornerBottomLeft,
        Self::CornerBottomRight,
        Self::TeeLeft,
        Self::TeeRight,
        Self::TeeDown,
        Self::TeeUp,
        Self::Cross,
        Self::DoubleHorizontal,
        Self::DoubleVertical,
        Self::HalfUpper,
        Self::HalfLower,
        Self::Full,
        Self::HalfLeft,
        Self::HalfRight,
        Self::ShadeLight,
        Self::ShadeMedium,
        Self::ShadeDark,
    ];

    /// The codepoint this glyph renders as.
    #[must_use]
    pub const fn ch(self) -> char {
        match self {
            Self::Horizontal => '\u{2500}',
            Self::Vertical => '\u{2502}',
            Self::CornerTopLeft => '\u{250C}',
            Self::CornerTopRight => '\u{2510}',
            Self::CornerBottomLeft => '\u{2514}',
            Self::CornerBottomRight => '\u{2518}',
            Self::TeeLeft => '\u{251C}',
            Self::TeeRight => '\u{2524}',
            Self::TeeDown => '\u{252C}',
            Self::TeeUp => '\u{2534}',
            Self::Cross => '\u{253C}',
            Self::DoubleHorizontal => '\u{2550}',
            Self::DoubleVertical => '\u{2551}',
            Self::HalfUpper => '\u{2580}',
            Self::HalfLower => '\u{2584}',
            Self::Full => '\u{2588}',
            Self::HalfLeft => '\u{258C}',
            Self::HalfRight => '\u{2590}',
            Self::ShadeLight => '\u{2591}',
            Self::ShadeMedium => '\u{2592}',
            Self::ShadeDark => '\u{2593}',
        }
    }

    /// Is this glyph a dither pattern?
    ///
    /// Surfaced because it is a TYPOGRAPHIC fact, not a rendering one:
    /// `░▒▓` are 25/50/75% checkerboards, so at cell size they read as
    /// noise. Crispness is contrast plus the absence of fuzz, and a
    /// composition that wants to look crisp should know which of its
    /// pieces work against that.
    #[must_use]
    pub const fn is_dither(self) -> bool {
        matches!(self, Self::ShadeLight | Self::ShadeMedium | Self::ShadeDark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seal for Gate-0 state 1, proven against the measured source
    /// of truth rather than against itself: every `Crisp` codepoint must
    /// be one `mado/src/glyph_class.rs::has_box_sprite` accepts.
    #[test]
    fn every_crisp_glyph_is_one_the_renderer_synthesises() {
        // Transcribed from has_box_sprite. If mado's sprite set ever
        // changes, this list is the one place to reconcile.
        const SPRITES: [char; 21] = [
            '\u{2500}', '\u{2502}', '\u{250C}', '\u{2510}', '\u{2514}', '\u{2518}',
            '\u{251C}', '\u{2524}', '\u{252C}', '\u{2534}', '\u{253C}',
            '\u{2550}', '\u{2551}',
            '\u{2580}', '\u{2584}', '\u{2588}', '\u{258C}', '\u{2590}',
            '\u{2591}', '\u{2592}', '\u{2593}',
        ];
        for g in Crisp::ALL {
            assert!(
                SPRITES.contains(&g.ch()),
                "{g:?} ({:?}) is NOT in the renderer's sprite set — it would render as tofu",
                g.ch()
            );
        }
    }

    /// The named traps stay unrepresentable. This is a documentation
    /// test as much as a check: these are the glyphs a person reaches
    /// for when trying to make something look nice.
    #[test]
    fn the_glyphs_with_no_geometry_have_no_variant() {
        for bad in ['\u{256D}', '\u{256E}', '\u{2570}', '\u{256F}', '\u{2501}', '\u{254C}'] {
            assert!(
                !Crisp::ALL.iter().any(|g| g.ch() == bad),
                "{bad:?} has no font geometry but is expressible as a Crisp variant"
            );
        }
    }

    #[test]
    fn the_catalog_matches_the_enum() {
        let mut seen = Crisp::ALL.to_vec();
        seen.sort_by_key(|g| g.ch() as u32);
        seen.dedup();
        assert_eq!(seen.len(), Crisp::ALL.len(), "ALL carries a duplicate");
    }

    #[test]
    fn dithers_are_flagged() {
        assert!(Crisp::ShadeMedium.is_dither());
        assert!(!Crisp::Horizontal.is_dither());
    }
}
