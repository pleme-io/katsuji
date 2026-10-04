//! `Attr` — the SGR attribute set, closed.
//!
//! A closed enum rather than a bitflag or a `u8`: an attribute that no
//! terminal implements has no variant, so it cannot be requested. The set
//! is deliberately SMALL — the widely-implemented attributes only.
//! Blink, conceal and the double-underline family are omitted because
//! their support is inconsistent enough that emitting them produces a
//! different result per terminal, which is the opposite of what a typed
//! emission surface is for.

use kazari::anstyle::Effects;

/// A character attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attr {
    /// SGR 1. Heavier weight — or a brighter colour on terminals that
    /// map bold to the bright palette.
    Bold,
    /// SGR 2. Reduced intensity. The workhorse for de-emphasis, and the
    /// reason a version number can sit beside a wordmark without
    /// competing with it.
    Dim,
    /// SGR 3.
    Italic,
    /// SGR 4.
    Underline,
    /// SGR 7. Swaps foreground and background.
    Reverse,
    /// SGR 9.
    Strike,
}

impl Attr {
    pub(crate) const fn effect(self) -> Effects {
        match self {
            Self::Bold => Effects::BOLD,
            Self::Dim => Effects::DIMMED,
            Self::Italic => Effects::ITALIC,
            Self::Underline => Effects::UNDERLINE,
            Self::Reverse => Effects::INVERT,
            Self::Strike => Effects::STRIKETHROUGH,
        }
    }

    /// Every attribute, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::Bold,
        Self::Dim,
        Self::Italic,
        Self::Underline,
        Self::Reverse,
        Self::Strike,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use kazari::anstyle;

    fn sgr(a: Attr) -> String {
        anstyle::Style::new().effects(a.effect()).render().to_string()
    }

    #[test]
    fn attributes_map_to_their_sgr_parameters() {
        assert_eq!(sgr(Attr::Bold), "\u{1b}[1m");
        assert_eq!(sgr(Attr::Dim), "\u{1b}[2m");
        assert_eq!(sgr(Attr::Italic), "\u{1b}[3m");
        assert_eq!(sgr(Attr::Underline), "\u{1b}[4m");
        assert_eq!(sgr(Attr::Reverse), "\u{1b}[7m");
        assert_eq!(sgr(Attr::Strike), "\u{1b}[9m");
    }

    #[test]
    fn no_attribute_collides_with_reset() {
        for a in Attr::ALL {
            assert!(!a.effect().is_plain(), "{a:?} sets no effect");
            assert_ne!(
                sgr(a),
                anstyle::Reset.render().to_string(),
                "{a:?} would emit a reset — every following piece would lose its style"
            );
        }
    }

    #[test]
    fn parameters_are_distinct() {
        let mut seen: Vec<String> = Attr::ALL.iter().map(|a| sgr(*a)).collect();
        seen.sort_unstable();
        let n = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), n, "two attributes share an SGR parameter");
    }
}
