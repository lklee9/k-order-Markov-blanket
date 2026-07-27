#import "@preview/peace-of-posters:0.6.0" as pop
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import "@preview/tiaoma:0.3.0"

// ===========================================================================
//  High-Order Markov Blanket Discovery — UAI 2026 poster
//  Fraunhofer IAIS · A0 portrait
//  Scaffold: theme + structure in place; section content filled in iteratively.
// ===========================================================================

// ---- Fraunhofer palette (from colors.xml) --------------------------------
#let fhg-green     = rgb("#179C7D")   // primary
#let fhg-blue      = rgb("#006E92")   // dark blue accent
#let fhg-lightblue = rgb("#25BAE2")
#let fhg-orange    = rgb("#EB6A0A")   // highlight
#let fhg-lime      = rgb("#B1C800")
#let fhg-grey      = rgb("#A8AFAF")
#let alert         = rgb("#C1272D")   // off-palette: marks a broken implication
#let fhg-bg        = rgb("#EDF0F0")   // page background (cards float on this)

// ---- Theme ----------------------------------------------------------------
#let fhg-theme = (
    "body-box-args": (
        inset: 0.7em,
        width: 100%,
        fill: white,
        stroke: none,
    ),
    "body-text-args": (
        fill: black,
    ),
    "heading-box-args": (
        inset: 0.5em,
        width: 100%,
        fill: none,
        stroke: none,
    ),
    "heading-text-args": (
        fill: fhg-blue,
        weight: "bold",
    ),
    "title-box-args": (
        inset: 1.9em,
        width: 100%,
        fill: fhg-green,
        stroke: none,
    ),
    "title-text-args": (
        fill: white,
        weight: "bold",
    ),
)

// ---- Page / layout setup ---------------------------------------------------
#set page("a0", margin: 1.6cm, fill: white)
#pop.set-poster-layout(pop.layout-a0)
#pop.update-poster-layout(
    body-size: 40pt,
    institutes-size: 40pt,
)
#pop.set-theme(fhg-theme)
#set text(size: pop.layout-a0.at("body-size"), fill: black, font: "Frutiger LT Com")
#let box-spacing = 1.2em
#set columns(gutter: box-spacing)
#set block(spacing: box-spacing)
#pop.update-poster-layout(spacing: box-spacing)

// ---- Small helpers ---------------------------------------------------------
// Placeholder marker for content we will write together. Remove as we go.
#let todo(body) = text(fill: fhg-grey, style: "italic")[#body]
// Inline highlight for key numbers / terms.
#let hl(body) = text(fill: fhg-orange, weight: "bold")[#body]
// Independence relation glyph (double up-tack) with an optional subscript.
#let ci(sub) = math.attach(math.class("relation", $perp#h(-0.42em)perp$), br: sub)
// ...and its negation, for stating dependence.
#let nci(sub) = math.attach(
    math.class("relation", math.cancel($perp#h(-0.42em)perp$)), br: sub,
)
// A figure panel: framed and white (no tint), so it reads as one self-contained
// unit against the body text without chopping the figure into sub-panels.
#let figure-block(body) = block(
    width: 100%,
    fill: white,
    stroke: 1.5pt + fhg-grey,
    radius: 8pt,
    inset: (x: 1em, y: 0.5em),
    body,
)

// ===========================================================================
//  TITLE
// ===========================================================================
// ---- Headline (betterposters style: state the MESSAGE, not the topic) -------
// Swap this one string to try another headline; the formal paper title stays as
// the subtitle underneath. Keep it <= ~12 words so it reads from 3 metres.
// NB "standard benchmarks", not "real networks": Alarm1/Barley/Insurance/Mildew
// supply the structure, but the samples are simulated — "real" would overclaim.
#let headline = "Higher-order faithfulness finds better Markov blankets — even on standard benchmarks"
#let formal-title = "High-Order Markov Blanket Discovery via a k-Order Relaxation of the Faithfulness Assumption"
#let qr-url = "https://github.com/lklee9/k-order-Markov-blanket"

#let qr-spacing = 2
#let authors = "Loong Kuan Lee, Ragavi Krishnamoorthy, Nico Piatkowski"
#let institutes = "Hybrid Intelligence · Fraunhofer IAIS · Germany"
#pop.title-box(
    headline,
    // subtitle: formal-title,
    authors: authors,
    institutes: institutes,
    logo: tiaoma.qrcode(qr-url, width: 100%, options: (
        bg-color: white,
        whitespace-width: qr-spacing,
        whitespace-height: qr-spacing)),    
)

// ===========================================================================
//  BODY
// ===========================================================================
#columns(2, [

    // ------------------------------------------------ LEFT COLUMN
    #pop.column-box(heading: "Motivation: Markov Blanket Discovery")[
        
        - *Markov blanket (MB) discovery* finds the minimal variable set
          that shields a target $Y$ from all other variables.

        - It is the backbone of feature selection, causal discovery, and
          Bayesian / Markov network structure learning.

        - Nearly all constraint-based methods rest on one load-bearing
          assumption, *faithfulness*.
      
    ]
    
    #pop.column-box(heading: "What is Faithfulness?")[
        
        Every independence in the distribution $P$ must appear as a
        separation in the graph $G$. 

        // One continuous figure (no sub-panels): the AND gate read two ways.
        // Colour, not chrome, carries the P-vs-G distinction: blue = distribution,
        // green = graph, orange = the faithfulness step between them.
        #let fig1 = figure-block[
            // Evidence outermost, independence statements innermost, so the two
            // sets of CIs sit either side of the arrow and the implication reads
            // straight across:  table | P-CIs  ==>  G-CIs | graph.
            // All columns sized to content (not 1fr, which would split the spare
            // width evenly and starve the wider CI column into wrapping), then
            // the whole row is centred in the panel.
            #align(center, grid(
                columns: (auto, auto, auto, auto, auto),
                column-gutter: 0.45em,
                row-gutter: (0.5em, 1.1em),   // wider gap between the two readings

                // ---- row 1: side labels, each spanning its own half ----
                grid.cell(colspan: 2,
                    align(center, text(fill: fhg-blue, weight: "bold", size: 0.78em)[
                        in the distribution #h(0.15em) $P$
                    ]),
                ),
                [],
                grid.cell(colspan: 2,
                    align(center, text(fill: fhg-green, weight: "bold", size: 0.78em)[
                        in the graph #h(0.15em) $G$
                    ]),
                ),

                // ---- row 2, col 1: the distribution as a truth table ----
                // Spans both reading directions (rows 2-3).
                grid.cell(rowspan: 2, align: horizon + center)[
                    #table(
                        columns: 3,
                        align: center,
                        inset: (x: 0.42em, y: 0.22em),
                        stroke: none,
                        table.hline(stroke: 1pt),
                        table.header([$X$], [$Z$], [$Y$]),
                        table.hline(stroke: 0.6pt),
                        [0], [0], [0],
                        [0], [1], [0],
                        [1], [0], [0],
                        [1], [1], [1],
                        table.hline(stroke: 1pt),
                    )
                ],

                // ---- row 2, col 2: the CIs it implies, right of the table ----
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$X #ci($P$) Z$]
                ],

                // ---- row 2, col 3: faithfulness ----
                grid.cell(align: horizon + center, stack(
                    text(size: 0.75em, fill: fhg-orange, weight: "bold")[faithfulness],
                    text(size: 2em, fill: fhg-orange, weight: "bold")[
                        $arrow.r.double.long$
                    ],
                )),

                // ---- row 2, col 4: the graph CIs, left of the graph ----
                // The line above already states the separation, so the note only
                // needs to give the reason — which keeps this column narrow.
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$X #ci($G$) Z$]
                ],

                // ---- row 2, col 5: the graph (spans rows 2-3) ----
                grid.cell(rowspan: 2, align: horizon + center)[
                    #diagram(
                        node-stroke: 1pt + black,
                        node-fill: white,
                        spacing: (2.1em, 1.8em),
                        node((0, 0), $X$, radius: 0.95em),
                        node((0.9, 0), $Z$, radius: 0.95em),
                        node((0.45, 0.85), $Y$, radius: 0.95em),
                        edge((0, 0), (0.45, 0.85), "-|>"),
                        edge((0.9, 0), (0.45, 0.85), "-|>"),
                    )
                ],

                // ---- row 3: the same assumption read backwards ----
                // Stated on an ADJACENT pair (X, Y): here the premise is true —
                // there is an edge — so the implication actually bites. This is
                // the direction MB discovery relies on, and the one XOR breaks.
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$X #nci($P$) Y$\ ]
                    #text(size: 1em)[$Z #nci($P$) Y$\ ]
                ],
                grid.cell(align: horizon + center, stack(
                    spacing: 0.25em,
                    text(size: 2em, fill: fhg-orange, weight: "bold")[
                        $arrow.l.double.long$
                    ],
                    text(size: 0.75em, fill: fhg-orange, weight: "bold")[
                        contra-\ positive
                    ],
                )),
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$X #nci($G$) Y$\ ]
                    #text(size: 1em)[$Z #nci($G$) Y$]
                ],
            ))
        ]
        
        #figure(
            fig1,
            numbering: none,
            caption: figure.caption([
                Example --- *AND gate* between
                independent fair coins.
            ], position: top)
        )

        Generally under the faithfulness assumption:
        #v(-0.75em)
        #align(center, box(
            fill: fhg-green.lighten(85%), inset: (x: 0.8em, y: 0.5em),
        )[
            $ X #ci($P$) Y mid(|) S quad ==> quad X #ci($G$) Y mid(|) S
            quad "for all disjoint" X, Y, S $
            $ X #nci($P$) Y mid(|) S quad <== quad X #nci($G$) Y mid(|) S
                quad "for all disjoint" X, Y, S $
        ])
    ]


    // ------------------------------------------------ RIGHT COLUMN
    #pop.column-box(heading: "When Faithfulness Breaks")[
        Either way, a true blanket member gets dropped.

        #v(0.2em)

        // ---------- (1) empirical violation: pathological sample ----------
        // The SAME noisy AND gate as fig1 — verified faithful in the true
        // distribution — but this 4-row sample (one noise flip, in red) makes
        // Y look exactly independent of X, while Z survives.
        #let fig2 = figure-block[
            #align(center, grid(
                columns: (auto, auto, auto, auto, auto),
                column-gutter: 0.45em,
                align: horizon,

                // the pathological sample
                grid.cell(align: horizon + center)[
                    #table(
                        columns: 3,
                        align: center,
                        inset: (x: 0.42em, y: 0.22em),
                        stroke: none,
                        table.hline(stroke: 1pt),
                        table.header([$X$], [$Z$], [$Y$]),
                        table.hline(stroke: 0.6pt),
                        [0], [0], [0],
                        [0], [1], text(fill: alert, weight: "bold")[1],
                        [1], [0], [0],
                        [1], [1], [1],
                        table.hline(stroke: 1pt),
                    )
                ],

                // what the sample reports
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$Y #ci($hat(P)$) X$]
                    #v(0.2em)
                    #text(size: 0.72em)[
                        $hat(P)(X, Y) = hat(P)(X) hat(P)(Y)$
                    ]
                ],

                // the contrapositive, broken again
                grid.cell(align: horizon + center, stack(
                    spacing: 0.25em,
                    text(size: 2em, fill: alert, weight: "bold")[
                        $arrow.l.double.not$
                    ],
                    text(size: 0.75em, fill: alert, weight: "bold")[
                        faithfulness\ fails
                    ],
                )),

                // what the graph says
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$Y #nci($G$) X$]
                ],

                // the graph: unchanged and still faithful
                grid.cell(align: horizon + center)[
                    #diagram(
                        node-stroke: 1pt + black,
                        node-fill: white,
                        spacing: (2.1em, 1.8em),
                        node((0, 0), $X$, radius: 0.95em),
                        node((0.9, 0), $Z$, radius: 0.95em),
                        node((0.45, 0.85), $Y$, radius: 0.95em),
                        edge((0, 0), (0.45, 0.85), "-|>"),
                        edge((0.9, 0), (0.45, 0.85), "-|>"),
                    )
                ],
            ))

            #v(0.25em)

            #align(center, text(size: 0.8em)[
                One noise flip makes $Y$ a copy of $Z$ — $X$ is dropped,
                $Z$ survives.
            ])
        ]

        #figure(
            fig2,
            numbering: none,
            caption: figure.caption([
                *Empirical* --- the *AND gate* is faithful, its *sample* is not.
            ], position: top)
        )

        // ---------- (2) structural violation: noisy XOR ----------
        // Same shape as fig1: the broken contrapositive read across, evidence in
        // the panel and the setup carried by a top caption.
        #let fig3 = figure-block[
            // Same five-column shape as fig1: table | P-CIs | arrow | G-CIs | graph.
            #align(center, grid(
                columns: (auto, auto, auto, auto, auto),
                column-gutter: 0.45em,
                align: horizon,

                // the XOR logic table
                grid.cell(align: horizon + center)[
                    #table(
                        columns: 3,
                        align: center,
                        inset: (x: 0.42em, y: 0.22em),
                        stroke: none,
                        table.hline(stroke: 1pt),
                        table.header([$X$], [$Z$], [$Y$]),
                        table.hline(stroke: 0.6pt),
                        [0], [0], [0],
                        [0], [1], [1],
                        [1], [0], [1],
                        [1], [1], [0],
                        table.hline(stroke: 1pt),
                    )
                ],

                // what every test in the data reports
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$Y #ci($P$) X$\ ]
                    #text(size: 1em)[$Y #ci($P$) Z$]
                ],

                // the contrapositive, broken
                grid.cell(align: horizon + center, stack(
                    spacing: 0.25em,
                    text(size: 2em, fill: alert, weight: "bold")[
                        $arrow.l.double.not$
                    ],
                    text(size: 0.75em, fill: alert, weight: "bold")[
                        faithfulness\ fails
                    ],
                )),

                // what the graph says
                grid.cell(align: horizon + center)[
                    #text(size: 1em)[$Y #nci($G$) X$\ ]
                    #text(size: 1em)[$Y #nci($G$) Z$]
                ],

                // the graph: same collider shape as fig1
                grid.cell(align: horizon + center)[
                    #diagram(
                        node-stroke: 1pt + black,
                        node-fill: white,
                        spacing: (2.1em, 1.8em),
                        node((0, 0), $X$, radius: 0.95em),
                        node((0.9, 0), $Z$, radius: 0.95em),
                        node((0.45, 0.85), $Y$, radius: 0.95em),
                        edge((0, 0), (0.45, 0.85), "-|>"),
                        edge((0.9, 0), (0.45, 0.85), "-|>"),
                    )
                ],
            ))

            #v(0.25em)

            #align(center, text(size: 0.8em)[
                Visible only given the other:#h(0.3em)
                $Y #nci($P$) X mid(|) Z$ — needs order $k = 1$.
            ])
        ]

        #figure(
            fig3,
            numbering: none,
            caption: figure.caption([
                *Structural* --- noisy *XOR*, $Y = X plus.o Z$ (w.p. $0.9$).
            ], position: top)
        )
    ]

    #colbreak()


    #pop.column-box(heading: [$k$-Order Faithfulness])[
        // NOTE: this section must carry the generalisation the XOR panel no
        // longer states — XOR is only the k=1 case, already handled by
        // 2-adjacency faithfulness [Marx+ 2021]; parity over k+2 variables
        // needs order k, and that is the novel part.
        #todo[$k$-order dependence + the assumption; XOR is only $k=1$,
        parity over $k+2$ variables needs order $k$.]
    ]

    #pop.column-box(heading: "The kOMB Algorithm")[
        #todo[Grow-and-Shrink, modified: search sets of up to $k$ variables +
        $l$-bounded separators. Correctness guarantee. One compact box.]
    ]

    #pop.column-box(heading: "Results")[
        #todo[Hero: synthetic F1 (parity 1.00 vs ~0.03) + syn_metrics_grid
        plot. Then benchmark BNs (Alarm1 / Barley / Insurance / Mildew):
        higher order helps on low-cardinality nets.]
    ]

    #pop.column-box(heading: "Takeaways & Future Work")[
        #todo[3-4 bullets: exploiting high-order dependence improves MB
        discovery; k = l ≤ 2 is a robust default; future = approximate /
        scalable variants.]
    ]

    // Nothing is cited yet, so this renders as a heading over an empty box and
    // just costs column height. Re-enable once the sections carry citations.
    // #pop.column-box(heading: "References")[
    //     #set text(size: 0.62em)
    //     #bibliography("bibliography.bib", title: none, style: "ieee")
    // ]


])

// ===========================================================================
//  BOTTOM
// ===========================================================================
#pop.bottom-box(
    heading-box-args: (inset: 1cm, fill: fhg-green),
    heading-text-args: (fill: white, weight: "bold")
)[
    #text(size: 0.8em)[
        // Paper Details:\
        #formal-title\
        #authors\
        #institutes\
        // Code: #link("https://github.com/lklee9/k-order-Markov-blanket")[github.com/lklee9/k-order-Markov-blanket]
    ]
    // Code: #link("https://github.com/lklee9/k-order-Markov-blanket")[github.com/lklee9/k-order-Markov-blanket]
    // #h(2em)
    // Funded by the German Federal Ministry of Research, Technology and Space and
    // the state of North Rhine-Westphalia as part of the Lamarr Institute for
    // Machine Learning and Artificial Intelligence.
]
