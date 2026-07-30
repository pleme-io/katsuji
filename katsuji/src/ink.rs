//! `Ink` — colour as a SEMANTIC SLOT, never a literal.
//!
//! # Gate 0, illegal state 2: a hardcoded hex colour
//!
//! Every fleet theme surface already states the rule — `mado/src/theme.rs`
//! repeats *"sourced from the ANSI cyan-frost slot (`ansi[6]`) — never a
//! hex"* on slot after slot, so an `ishou` retune propagates on the next
//! compile. But nothing ENFORCED it: a caller could always write
//! `"\x1b[38;2;136;192;208m"` and pin a colour that stops tracking the
//! theme the moment it is retuned.
//!
//! Here the rule is the type. [`Ink`] carries a slot, and there is no
//! `Ink::from_hex`, no `Ink::Rgb`, no `impl From<(u8,u8,u8)>`. A literal
//! colour has no constructor, so it cannot reach a rendered line — a
//! compile error, not a review comment.
//!
//! The resolution from slot to pixels stays where it belongs: the
//! terminal's own palette, or `ishou`/`irodori` when a GPU consumer needs
//! the concrete value. Katsuji emits the INDEX and lets the theme decide,
//! which is precisely why a retune needs no change here.

/// A terminal colour, addressed by its semantic slot.
///
/// The variants are the ANSI-16 roles plus `Default`. Naming them by ROLE
/// rather than by appearance is deliberate: `Ink::Cyan` under Nord is
/// frost `#88C0D0` and under Vellum is something else, and a banner drawn
/// in `Ink::Cyan` is correct in both without knowing either.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ink {
    /// The terminal's configured foreground — SGR 39.
    Default,
    Black,
    Red,
    Green,
    Yellow,
    /// `ansi[4]`. The fleet's frost-blue accent under Nord.
    Blue,
    Magenta,
    /// `ansi[6]`. The fleet's frost-cyan accent under Nord — the colour
    /// `theme.rs` names for links, prompt marks and the scrollbar.
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Ink {
    /// The SGR parameter for this ink in the given position.
    ///
    /// Private to the crate: a parameter number is an implementation
    /// detail of rendering, and exposing it would hand a caller the
    /// pieces to hand-assemble an escape — the thing this crate exists to
    /// make unnecessary.
    pub(crate) const fn sgr_param(self, position: Position) -> u8 {
        let base = match position {
            Position::Foreground => 30,
            Position::Background => 40,
        };
        match self {
            Self::Default => base + 9,
            Self::Black => base,
            Self::Red => base + 1,
            Self::Green => base + 2,
            Self::Yellow => base + 3,
            Self::Blue => base + 4,
            Self::Magenta => base + 5,
            Self::Cyan => base + 6,
            Self::White => base + 7,
            // The bright range sits 60 above its normal counterpart in
            // both positions (90-97 / 100-107).
            Self::BrightBlack => base + 60,
            Self::BrightRed => base + 61,
            Self::BrightGreen => base + 62,
            Self::BrightYellow => base + 63,
            Self::BrightBlue => base + 64,
            Self::BrightMagenta => base + 65,
            Self::BrightCyan => base + 66,
            Self::BrightWhite => base + 67,
        }
    }
}

/// Whether an ink paints the glyph or the cell behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Position {
    Foreground,
    Background,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_map_to_their_ansi_parameters() {
        assert_eq!(Ink::Cyan.sgr_param(Position::Foreground), 36);
        assert_eq!(Ink::Blue.sgr_param(Position::Foreground), 34);
        assert_eq!(Ink::Default.sgr_param(Position::Foreground), 39);
        assert_eq!(Ink::Cyan.sgr_param(Position::Background), 46);
        assert_eq!(Ink::BrightCyan.sgr_param(Position::Foreground), 96);
        assert_eq!(Ink::BrightWhite.sgr_param(Position::Background), 107);
    }

    /// The seal for Gate-0 state 2, asserted as a property of the API
    /// rather than a comment: every way to obtain an `Ink` is a named
    /// slot. If someone adds `Ink::Rgb(u8,u8,u8)` or a `from_hex`, this
    /// test still compiles — so it is backed by the doc contract above
    /// AND by the enum having no payload-carrying variant. The real
    /// enforcement is that `Ink` is a fieldless enum: there is nowhere
    /// to put a literal.
    #[test]
    fn every_ink_is_a_slot_not_a_literal() {
        // A fieldless enum is Copy + constructible only by naming a
        // variant. This compiles only while that stays true.
        const _: fn() = || {
            let _: Ink = Ink::Cyan;
        };
        assert_eq!(std::mem::size_of::<Ink>(), 1, "Ink grew a payload — a literal colour just became representable");
    }
}
