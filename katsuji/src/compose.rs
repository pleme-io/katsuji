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
//! **4 — a raw `\x1b` from a consumer.** No escape is spelled here at
//! all: every byte comes from kazari's `StyleAtom`, so the fleet has one
//! emitter. A consumer composes `Piece`s; it is never
//! handed an escape to concatenate. `format!()` of an escape is what ★★
//! TYPED EMISSION bans, and this is the typed surface that makes it
//! unnecessary.
//!
//! The capability wall is kazari's: [`Line::render_at`] takes a
//! `kazari::Capability`, and at `ColorLevel::None` it is the plain
//! projection. [`Line::render`] stays the unconditional ANSI-16 form.
//!
//! # What this is NOT
//!
//! It is not a layout engine. It composes ONE line at a time from typed
//! pieces. Multi-line art is a `Vec<Line>` and stays the caller's shape —
//! deliberately, because layout belongs to the consumer (a banner, a
//! status bar, a TUI widget) while typed emission belongs here.

use kazari::anstyle::{Color, Effects};
use kazari::style::StyleAtom;
use kazari::{Capability, ColorLevel, Stream};

use crate::ink::Ink;
use crate::sgr::Attr;

const ANSI16: Capability = Capability::fixed(ColorLevel::Ansi16, 80, true);

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

    fn atom(&self, caps: &Capability) -> StyleAtom {
        StyleAtom::styled(
            self.ink.ansi().map(Color::Ansi),
            self.on.and_then(Ink::ansi).map(Color::Ansi),
            self.attrs.iter().fold(Effects::new(), |e, a| e | a.effect()),
            caps,
        )
    }

    fn write_content(&self, out: &mut String) {
        match &self.content {
            Content::Text(s) => out.push_str(s),
            Content::Glyphs(g, n) => {
                for _ in 0..*n {
                    out.push(g.ch());
                }
            }
        }
    }

    fn write_into(&self, out: &mut String, caps: &Capability) {
        let mut content = String::new();
        self.write_content(&mut content);
        out.push_str(&self.atom(caps).paint(&content));
    }
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
    /// the `MoonScript` quality is largely whitespace — so it is a verb
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
        self.render_at(&ANSI16)
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
            p.write_content(&mut out);
        }
        out
    }

    #[must_use]
    pub fn render_at(&self, caps: &Capability) -> String {
        let mut out = String::new();
        for p in &self.pieces {
            p.write_into(&mut out, caps);
        }
        out
    }

    #[must_use]
    pub fn render_for(&self, stream: Stream) -> String {
        self.render_at(&Capability::probe_stream(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::Crisp;
    use kazari::{Role, Theme};

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
    fn sgr_params_are_emitted_in_order() {
        let s = Line::new()
            .piece(Piece::text("x").ink(Ink::Cyan).on(Ink::Black).attr(Attr::Bold))
            .render();
        assert_eq!(s, "\u{1b}[1m\u{1b}[36m\u{1b}[40mx\u{1b}[0m", "unexpected SGR: {s:?}");
    }

    fn sample() -> Line {
        Line::new()
            .piece(Piece::glyphs(Crisp::Horizontal, 3).ink(Ink::BrightCyan).attr(Attr::Dim))
            .gap(1)
            .piece(Piece::text("mado").ink(Ink::Blue).on(Ink::Black).attr(Attr::Bold))
    }

    #[test]
    fn the_capability_floor_is_the_plain_projection() {
        let l = sample();
        assert_eq!(l.render_at(&Capability::plain()), l.plain());
    }

    #[test]
    fn every_coloured_rung_emits_the_ansi_slot_not_a_literal() {
        let l = sample();
        for level in ColorLevel::ALL.into_iter().filter(|l| l.is_colored()) {
            let s = l.render_at(&Capability::fixed(level, 80, true));
            assert_eq!(s, l.render(), "{level:?} changed the bytes");
            assert!(!s.contains("38;2;") && !s.contains("38;5;"), "{level:?} pinned a literal: {s:?}");
        }
    }

    #[test]
    fn a_piece_with_only_a_default_background_emits_nothing() {
        let s = Line::new().piece(Piece::text("x").on(Ink::Default)).render();
        assert_eq!(s, "x");
    }

    #[test]
    fn a_role_ink_renders_byte_for_byte_as_kazari_paints_the_role() {
        let caps = Capability::fixed(ColorLevel::Ansi16, 80, true);
        for role in Role::ALL {
            for (bold, dim) in [(false, false), (true, false), (false, true)] {
                let mut piece = Piece::text("kazari").ink(Ink::from(role));
                if bold {
                    piece = piece.attr(Attr::Bold);
                }
                if dim {
                    piece = piece.attr(Attr::Dim);
                }
                let ours = Line::new().piece(piece).render();
                let theirs = StyleAtom::resolve(role, Theme::default(), &caps, bold, dim).paint("kazari");
                assert_eq!(ours, theirs, "{role:?} bold={bold} dim={dim} diverged from kazari");
            }
        }
    }
}
