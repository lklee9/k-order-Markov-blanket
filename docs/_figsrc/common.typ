// Shared preamble for the project page's figures.
//
// These are the POSTER's figures, re-exported for the web from the same code as
// poster/main.typ. Keep the definitions below in sync with the top of that file;
// the underscore on this directory keeps Jekyll from publishing the sources.
//
// Two things here are load-bearing and easy to break:
//
//   * `margin` must stay > 0pt. At 0pt Typst crops to the layout box and shaves
//     the node strokes (measured: 59.6x54.2pt instead of 67.6x62.2pt).
//   * `fill: none` must stay. The default bakes an opaque white rectangle over
//     the whole canvas as the first drawing op, which defeats the page's own
//     figure background and looks wrong on any tinted card.
//
// Text size stays at the poster's 40pt even though these render ~100px wide:
// the geometry is cm-based with absolute pt strokes, so shrinking the type here
// would change the label-to-stroke ratio. Scale in CSS instead.
//
// Frutiger LT Com must be installed on whatever machine builds these. A missing
// font is only a WARNING with exit code 0 — Typst silently substitutes and the
// metrics shift — so build.sh greps stderr and fails on it.

#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import "@preview/cetz:0.3.4"

#let setup(body) = {
    set page(width: auto, height: auto, margin: 6pt, fill: none)
    set text(size: 40pt, font: "Frutiger LT Com", fill: black)
    set par(justify: false)
    body
}

// ---- Fraunhofer palette (verbatim from poster/main.typ) ---------------------
#let fhg-green  = rgb("#179C7D")
#let fhg-orange = rgb("#EB6A0A")
#let fhg-grey   = rgb("#A8AFAF")
#let grey-dark  = rgb("#6E7676")

// ---- the collider, as drawn in the poster's faithfulness panel --------------
// poster/main.typ:368-377
#let collider = diagram(
    node-stroke: 1pt + black,
    node-fill: white,
    spacing: (2.1em, 1.8em),
    node((0, 0), $X$, radius: 0.95em),
    node((0.9, 0), $Z$, radius: 0.95em),
    node((0.45, 0.85), $Y$, radius: 0.95em),
    edge((0, 0), (0.45, 0.85), "-|>"),
    edge((0.9, 0), (0.45, 0.85), "-|>"),
)

// ---- one rung of the k-order ladder ----------------------------------------
// poster/main.typ:586-605. `n-par` counts the parents of Y: the tested variable
// X (green) plus n-par - 1 witnesses (orange). So n-par = 1, 2, 3 gives the
// k = 0, k = 1 and k = 2 rungs.
//
// These are exported as three separate files and sized by HEIGHT in CSS, never
// by width: the rungs hold 2, 3 and 4 nodes, so width-sizing would render the
// k = 0 circles at twice the diameter of the k = 2 circles and destroy the
// "same graph, more witnesses" comparison that is the whole point. Height-sized,
// the widths grow 1:2:3, which IS the visual argument.
#let rung(n-par) = {
    let R = 0.85em
    let xs = range(n-par).map(i => i * 1.0)
    let cx = (xs.at(0) + xs.at(-1)) / 2
    diagram(
        node-stroke: 1pt + black,
        node-fill: white,
        spacing: (2.1em, 1.5em),
        node((0, 0), $X$, radius: R, stroke: 1.6pt + fhg-green),
        edge((0, 0), (cx, 1), "-|>", stroke: 1.6pt + fhg-green),
        ..range(1, n-par).map(i => node((xs.at(i), 0),
            text(fill: fhg-orange)[$W_#i$],
            radius: R, stroke: 1.6pt + fhg-orange,
        )),
        ..range(1, n-par).map(i => edge((xs.at(i), 0), (cx, 1), "-|>")),
        node((cx, 1), $Y$, radius: R),
    )
}
