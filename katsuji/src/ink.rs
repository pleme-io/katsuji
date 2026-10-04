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

use anstyle::AnsiColor;

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
    pub(crate) const fn ansi(self) -> Option<AnsiColor> {
        match self {
            Self::Default => None,
            Self::Black => Some(AnsiColor::Black),
            Self::Red => Some(AnsiColor::Red),
            Self::Green => Some(AnsiColor::Green),
            Self::Yellow => Some(AnsiColor::Yellow),
            Self::Blue => Some(AnsiColor::Blue),
            Self::Magenta => Some(AnsiColor::Magenta),
            Self::Cyan => Some(AnsiColor::Cyan),
            Self::White => Some(AnsiColor::White),
            Self::BrightBlack => Some(AnsiColor::BrightBlack),
            Self::BrightRed => Some(AnsiColor::BrightRed),
            Self::BrightGreen => Some(AnsiColor::BrightGreen),
            Self::BrightYellow => Some(AnsiColor::BrightYellow),
            Self::BrightBlue => Some(AnsiColor::BrightBlue),
            Self::BrightMagenta => Some(AnsiColor::BrightMagenta),
            Self::BrightCyan => Some(AnsiColor::BrightCyan),
            Self::BrightWhite => Some(AnsiColor::BrightWhite),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use kazari::Role;

    pub(crate) const fn from_ansi(slot: AnsiColor) -> Ink {
        match slot {
            AnsiColor::Black => Ink::Black,
            AnsiColor::Red => Ink::Red,
            AnsiColor::Green => Ink::Green,
            AnsiColor::Yellow => Ink::Yellow,
            AnsiColor::Blue => Ink::Blue,
            AnsiColor::Magenta => Ink::Magenta,
            AnsiColor::Cyan => Ink::Cyan,
            AnsiColor::White => Ink::White,
            AnsiColor::BrightBlack => Ink::BrightBlack,
            AnsiColor::BrightRed => Ink::BrightRed,
            AnsiColor::BrightGreen => Ink::BrightGreen,
            AnsiColor::BrightYellow => Ink::BrightYellow,
            AnsiColor::BrightBlue => Ink::BrightBlue,
            AnsiColor::BrightMagenta => Ink::BrightMagenta,
            AnsiColor::BrightCyan => Ink::BrightCyan,
            AnsiColor::BrightWhite => Ink::BrightWhite,
        }
    }

    pub(crate) fn every_role() -> [Role; 11] {
        let roles = [
            Role::Primary,
            Role::Text,
            Role::TextMuted,
            Role::TextDim,
            Role::Border,
            Role::Error,
            Role::Warn,
            Role::Pending,
            Role::Ok,
            Role::Info,
            Role::Ident,
        ];
        for role in roles {
            match role {
                Role::Primary
                | Role::Text
                | Role::TextMuted
                | Role::TextDim
                | Role::Border
                | Role::Error
                | Role::Warn
                | Role::Pending
                | Role::Ok
                | Role::Info
                | Role::Ident => {}
            }
        }
        roles
    }

    fn fg(ink: Ink) -> String {
        anstyle::Style::new().fg_color(ink.ansi().map(anstyle::Color::Ansi)).render().to_string()
    }

    fn bg(ink: Ink) -> String {
        anstyle::Style::new().bg_color(ink.ansi().map(anstyle::Color::Ansi)).render().to_string()
    }

    #[test]
    fn slots_map_to_their_ansi_parameters() {
        assert_eq!(fg(Ink::Cyan), "\u{1b}[36m");
        assert_eq!(fg(Ink::Blue), "\u{1b}[34m");
        assert_eq!(fg(Ink::Default), "");
        assert_eq!(bg(Ink::Cyan), "\u{1b}[46m");
        assert_eq!(fg(Ink::BrightCyan), "\u{1b}[96m");
        assert_eq!(bg(Ink::BrightWhite), "\u{1b}[107m");
    }

    #[test]
    fn every_slot_round_trips_through_its_ansi_colour() {
        for slot in [
            Ink::Black,
            Ink::Red,
            Ink::Green,
            Ink::Yellow,
            Ink::Blue,
            Ink::Magenta,
            Ink::Cyan,
            Ink::White,
            Ink::BrightBlack,
            Ink::BrightRed,
            Ink::BrightGreen,
            Ink::BrightYellow,
            Ink::BrightBlue,
            Ink::BrightMagenta,
            Ink::BrightCyan,
            Ink::BrightWhite,
        ] {
            let ansi = slot.ansi().expect("a named slot has an ANSI colour");
            assert_eq!(from_ansi(ansi), slot);
        }
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
