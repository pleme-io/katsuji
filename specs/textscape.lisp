;; katsuji — the textscape authoring surface.
;;
;; ─── STATUS: THIS IS THE M1 DESTINATION, NOT THE M0 WIRED FORM ────────
;;
;; M0 is typed Rust (katsuji/src/{ink,sgr,glyph,compose}.rs). This file
;; DOCUMENTS the (deftexto ...) form that #[derive(DeriveTataraDomain)]
;; will compile at M1. A documented form is not a wired form, and saying
;; otherwise is the tier-rounding the vocabularify skill forbids.
;;
;; What M1 buys: a banner, a status line or a CLI's whole output becomes a
;; DECLARATION rather than Rust. A new one is an entry here, not a build.
;;
;; ─── WHY THE FIELDS ARE SHAPED THIS WAY ───────────────────────────────
;;
;; :ink takes a SLOT NAME, never a hex. That is Gate-0 illegal state 2,
;; and it is the same rule mado's theme.rs states on every slot ("never a
;; hex"), except here the reader cannot express the violation: there is no
;; hex syntax in the vocabulary to write.
;;
;; :glyph takes a ROLE NAME from the crisp set, never a codepoint. Gate-0
;; state 1. `corner-top-left` is expressible; `rounded-corner-top-left` is
;; not, because the glyph it names has no font geometry and would render
;; as tofu. Naming by role also means a composition reads as intent.
;;
;; There is NO :reset and NO :open/:close. Gate-0 state 3: a piece carries
;; its style, and the renderer emits both ends. An author has no way to
;; leave a style open, because they never open one.

(deftexto
  :name "mado-banner"
  :doc "mado's startup banner — a window (窓), drawn in the crisp set."

  ;; Each line is a list of pieces. A piece is (:glyph ROLE :times N ...)
  ;; or (:text "..." ...), plus optional :ink / :on / :attrs.
  :lines
  [[(:glyph corner-top-left :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph tee-down :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph corner-top-right :ink cyan)]

   [(:glyph vertical :ink cyan)
    (:gap 3)
    (:glyph vertical :ink cyan)
    (:gap 3)
    (:glyph vertical :ink cyan)
    (:gap 4)
    (:text "mado" :attrs [bold])
    (:gap 2)
    ;; The version is INTERPOLATED from the package at render time, never
    ;; written here. A literal would be a second place to update and would
    ;; drift — exactly the defect that had the installed Mado.app
    ;; reporting 0.1.0 against a 0.1.98 binary.
    (:text (:from-package version) :attrs [dim])]

   [(:glyph tee-left :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph cross :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph tee-right :ink cyan)]

   [(:glyph vertical :ink cyan)
    (:gap 3)
    (:glyph vertical :ink cyan)
    (:gap 3)
    (:glyph vertical :ink cyan)
    (:gap 4)
    (:text "gpu terminal" :attrs [dim])]

   [(:glyph corner-bottom-left :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph tee-up :ink cyan)
    (:glyph horizontal :times 3 :ink cyan)
    (:glyph corner-bottom-right :ink cyan)]])
