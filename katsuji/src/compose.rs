//! Composing a line from typed pieces — and the ONE place escapes exist.
//!
//! # Gate 0, illegal states 3 and 4
//!
//! **3 — unbalanced SGR.** The classic terminal bug is `\x1b[1m` written
//! without its `\x1b[0m`, so bold bleeds into everything after it,
//! including the user's prompt. It happens because "set an attribute" and
//! "clear it" are two separate emissions and nothing pairs them.
//!
//! Here there is no verb that opens a style. A [`Piece`] CARRIES its
//! style, and [`render`] emits the reset as part of writing the piece.
//! Bleed is not guarded against — it has no way to occur, because the
//! open and the close are produced by one line of code that a caller
//! never sees.
//!
//! **4 — a raw `\x1b` from a consumer.** Escapes are constructed in
//! exactly one private function in this module. A consumer composes
//! `Piece`s; it is never handed an escape to concatenate. `format!()` of
//! an escape is what ★★ TYPED EMISSION bans, and this is the typed
//! surface that makes it unnecessary.
//!
//! # What this is NOT
//!
//! It is not a layout engine. It composes ONE line at a time from typed
//! pieces. Multi-line art is a `Vec<Line>` and stays the caller's shape —
//! deliberately, because layout belongs to the consumer (a banner, a
//! status bar, a TUI widget) while typed emission belongs here.

use crate::ink::{Ink, Position};
use crate::sgr::Attr;

/// A styled run of content.
///
/// Style travels WITH the content rather than being switched on around
/// it — that is what makes an unbalanced attribute unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    content: Content,
    ink: Ink,
    on: Option<Ink>,
    attrs: Vec<Attr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Content {
    /// Literal text. Carried verbatim — this crate does not own prose.
    Text(String),
    /// A glyph guaranteed to render, repeated `n` times.
    Glyphs(crate::glyph::Crisp, usize),
}

impl Piece {
    /// A piece of literal text, unstyled.
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self { content: Content::Text(s.into()), ink: Ink::Default, on: None, attrs: Vec::new() }
    }

    /// `n` repetitions of a crisp glyph — a rule, a fill, a frame edge.
    #[must_use]
    pub fn glyphs(g: crate::glyph::Crisp, n: usize) -> Self {
        Self { content: Content::Glyphs(g, n), ink: Ink::Default, on: None, attrs: Vec::new() }
    }

    /// One crisp glyph.
    #[must_use]
    pub fn glyph(g: crate::glyph::Crisp) -> Self {
        Self::glyphs(g, 1)
    }

    /// Paint the glyphs in a semantic slot.
    #[must_use]
    pub fn ink(mut self, ink: Ink) -> Self {
        self.ink = ink;
        self
    }

    /// Paint the cell behind the glyphs.
    #[must_use]
    pub fn on(mut self, ink: Ink) -> Self {
        self.on = Some(ink);
        self
    }

    /// Add an attribute. Idempotent — asking twice does not double-emit.
    #[must_use]
    pub fn attr(mut self, a: Attr) -> Self {
        if !self.attrs.contains(&a) {
            self.attrs.push(a);
        }
        self
    }

    /// The printable width of this piece in terminal cells.
    ///
    /// # Gate 0, illegal state 5 (eval-caught, stated honestly)
    ///
    /// Width is COMPUTED here, never asserted by the caller, so a
    /// composition cannot claim a width it does not have. But the
    /// computation is only correct for the content this crate can reason
    /// about: crisp glyphs are all single-width by construction, and
    /// `Text` is counted in `char`s.
    ///
    /// That char count is WRONG for CJK and emoji, which are double-width,
    /// and for combining marks, which are zero-width. Katsuji does not
    /// vendor a width table at M0, so alignment of such text is the
    /// caller's problem and this returns a lower bound. Said plainly
    /// rather than rounded up: this is *eval-caught*, not
    /// truly-unrepresentable — a real `unicode-width` seam is M1.
    #[must_use]
    pub fn width(&self) -> usize {
        match &self.content {
            Content::Text(s) => s.chars().count(),
            Content::Glyphs(_, n) => *n,
        }
    }

    fn write_into(&self, out: &mut String) {
        let styled = self.ink != Ink::Default || self.on.is_some() || !self.attrs.is_empty();
        if styled {
            let mut params: Vec<u8> = Vec::with_capacity(self.attrs.len() + 2);
            for a in &self.attrs {
                params.push(a.sgr_param());
            }
            if self.ink != Ink::Default {
                params.push(self.ink.sgr_param(Position::Foreground));
            }
            if let Some(bg) = self.on {
                params.push(bg.sgr_param(Position::Background));
            }
            push_sgr(out, &params);
        }
        match &self.content {
            Content::Text(s) => out.push_str(s),
            Content::Glyphs(g, n) => {
                for _ in 0..*n {
                    out.push(g.ch());
                }
            }
        }
        if styled {
            // The close is emitted HERE, unconditionally, by the same
            // code that emitted the open. There is no path that writes
            // one without the other.
            push_sgr(out, &[Attr::RESET]);
        }
    }
}

/// THE ONLY PLACE AN ESCAPE SEQUENCE IS CONSTRUCTED.
///
/// Private, and deliberately the single point of contact with the byte
/// `0x1b`. Every rendered attribute in the fleet routes through these
/// three lines, so the wire format is one thing to get right and one
/// thing to change.
fn push_sgr(out: &mut String, params: &[u8]) {
    out.push_str("\u{1b}[");
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            out.push(';');
        }
        // A u8 renders as at most 3 ASCII digits; no allocation needed.
        let mut buf = [0u8; 3];
        let mut n = *p;
        let mut len = 0;
        loop {
            buf[len] = b'0' + (n % 10);
            n /= 10;
            len += 1;
            if n == 0 {
                break;
            }
        }
        for k in (0..len).rev() {
            out.push(buf[k] as char);
        }
    }
    out.push('m');
}

/// A composed line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Line {
    pieces: Vec<Piece>,
}

impl Line {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a piece.
    #[must_use]
    pub fn piece(mut self, p: Piece) -> Self {
        self.pieces.push(p);
        self
    }

    /// Append `n` spaces. Spacing is a first-class compositional act —
    /// the MoonScript quality is largely whitespace — so it is a verb
    /// rather than a `Piece::text("   ")` incantation.
    #[must_use]
    pub fn gap(self, n: usize) -> Self {
        self.piece(Piece::text(" ".repeat(n)))
    }

    /// Printable width, ignoring escapes. See [`Piece::width`] for the
    /// honest limits of this number.
    #[must_use]
    pub fn width(&self) -> usize {
        self.pieces.iter().map(Piece::width).sum()
    }

    /// Render to a terminal-ready string.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        for p in &self.pieces {
            p.write_into(&mut out);
        }
        out
    }

    /// Render with every escape omitted — the plain-text projection.
    ///
    /// This is what makes a composition TESTABLE and pipe-safe: the same
    /// typed line yields styled bytes for a TTY and clean text for a log
    /// or a file, with no second code path to keep in sync.
    #[must_use]
    pub fn plain(&self) -> String {
        let mut out = String::new();
        for p in &self.pieces {
            match &p.content {
                Content::Text(s) => out.push_str(s),
                Content::Glyphs(g, n) => {
                    for _ in 0..*n {
                        out.push(g.ch());
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::Crisp;

    /// The seal for Gate-0 state 3: every emitted open has its close.
    #[test]
    fn a_styled_piece_always_closes_itself() {
        let s = Line::new()
            .piece(Piece::text("mado").ink(Ink::Cyan).attr(Attr::Bold))
            .render();
        assert!(s.starts_with("\u{1b}["), "no opening SGR: {s:?}");
        assert!(s.ends_with("\u{1b}[0m"), "style was left open — it will bleed: {s:?}");
    }

    #[test]
    fn styles_never_bleed_between_pieces() {
        let s = Line::new()
            .piece(Piece::text("a").ink(Ink::Cyan))
            .piece(Piece::text("b"))
            .render();
        // "b" is unstyled, so it must sit AFTER a reset with no opener.
        let after_reset = s.split("\u{1b}[0m").nth(1).expect("a reset");
        assert_eq!(after_reset, "b", "unstyled content inherited a style: {s:?}");
    }

    #[test]
    fn an_unstyled_piece_emits_no_escapes_at_all() {
        let s = Line::new().piece(Piece::text("plain")).render();
        assert_eq!(s, "plain", "escapes emitted for unstyled content: {s:?}");
    }

    #[test]
    fn the_plain_projection_has_no_escapes() {
        let l = Line::new()
            .piece(Piece::glyphs(Crisp::Horizontal, 3).ink(Ink::Cyan))
            .gap(1)
            .piece(Piece::text("mado").attr(Attr::Bold));
        assert_eq!(l.plain(), "─── mado");
        assert!(!l.plain().contains('\u{1b}'), "plain leaked an escape");
        assert!(l.render().contains('\u{1b}'), "render lost its styling");
    }

    #[test]
    fn width_ignores_styling() {
        let l = Line::new()
            .piece(Piece::glyphs(Crisp::Horizontal, 9).ink(Ink::Cyan).attr(Attr::Bold))
            .gap(2)
            .piece(Piece::text("mado"));
        assert_eq!(l.width(), 15);
        assert_eq!(l.width(), l.plain().chars().count());
    }

    #[test]
    fn attributes_are_idempotent() {
        let once = Line::new().piece(Piece::text("x").attr(Attr::Bold)).render();
        let twice = Line::new().piece(Piece::text("x").attr(Attr::Bold).attr(Attr::Bold)).render();
        assert_eq!(once, twice, "a repeated attribute double-emitted");
    }

    #[test]
    fn sgr_params_are_joined_correctly() {
        let s = Line::new()
            .piece(Piece::text("x").ink(Ink::Cyan).on(Ink::Black).attr(Attr::Bold))
            .render();
        assert!(s.starts_with("\u{1b}[1;36;40m"), "unexpected SGR: {s:?}");
    }
}
