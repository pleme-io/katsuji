//! `Attr` — the SGR attribute set, closed.
//!
//! A closed enum rather than a bitflag or a `u8`: an attribute that no
//! terminal implements has no variant, so it cannot be requested. The set
//! is deliberately SMALL — the widely-implemented attributes only.
//! Blink, conceal and the double-underline family are omitted because
//! their support is inconsistent enough that emitting them produces a
//! different result per terminal, which is the opposite of what a typed
//! emission surface is for.

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
    /// SGR 0 — clear everything. Not a variant: resetting is not an
    /// attribute a caller applies to content, it is what
    /// [`crate::compose`] emits to close a piece. Keeping it out of the
    /// enum is what stops a caller "resetting" mid-line and re-creating
    /// the unbalanced-style bug by hand.
    pub(crate) const RESET: u8 = 0;

    pub(crate) const fn sgr_param(self) -> u8 {
        match self {
            Self::Bold => 1,
            Self::Dim => 2,
            Self::Italic => 3,
            Self::Underline => 4,
            Self::Reverse => 7,
            Self::Strike => 9,
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

    #[test]
    fn attributes_map_to_their_sgr_parameters() {
        assert_eq!(Attr::Bold.sgr_param(), 1);
        assert_eq!(Attr::Dim.sgr_param(), 2);
        assert_eq!(Attr::Strike.sgr_param(), 9);
    }

    #[test]
    fn no_attribute_collides_with_reset() {
        for a in Attr::ALL {
            assert_ne!(
                a.sgr_param(),
                Attr::RESET,
                "{a:?} would emit a reset — every following piece would lose its style"
            );
        }
    }

    #[test]
    fn parameters_are_distinct() {
        let mut seen: Vec<u8> = Attr::ALL.iter().map(|a| a.sgr_param()).collect();
        seen.sort_unstable();
        let n = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), n, "two attributes share an SGR parameter");
    }
}
