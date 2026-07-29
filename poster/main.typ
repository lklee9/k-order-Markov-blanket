#import "@preview/peace-of-posters:0.6.0" as pop
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
#import "@preview/tiaoma:0.3.0"
#import "@preview/cetz:0.3.4"

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
        inset: (left: 2em, right: 2em, top: 2em, bottom: 2em),
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
    title-size: 100pt,
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
// "does not imply", pointing down. Typst has no negated vertical double arrow,
// so the clean horizontal glyph is rotated rather than built out of cancel()
// (which renders as an illegible knot of strokes at poster sizes).
#let nimplies-down = rotate(-90deg, reflow: true)[$arrow.l.double.not$]
// A figure panel: framed and white (no tint), so it reads as one self-contained
// unit against the body text without chopping the figure into sub-panels.
#let figure-block(body) = block(
    width: 100%,
    fill: white,
    stroke: 1.5pt + fhg-grey,
    radius: 8pt,
    inset: (x: 0.8em, y: 0.4em),
    body,
)

// ===========================================================================
//  TITLE
// ===========================================================================
// ---- Headline (betterposters style: state the MESSAGE, not the topic) -------
// Swap this one string to try another headline; the formal paper title sits in
// the identity strip below. Keep it <= ~12 words so it reads from 3 metres.
// NB "standard benchmarks", not "real networks": Alarm1/Barley/Insurance/Mildew
// supply the structure, but the samples are simulated — "real" would overclaim.
#let headline = [Higher-order faithfulness finds better Markov blankets — even on standard benchmarks]
#let formal-title = "High-Order Markov Blanket Discovery via a k-Order Relaxation of the Faithfulness Assumption"
#let qr-url = "https://github.com/lklee9/k-order-Markov-blanket"

#let qr-spacing = 4   // ISO/IEC 18004 minimum quiet zone
// One source of truth for the names: `authors` is the plain footer list,
// `authors-lead` bolds the presenting author for the identity strip.
#let author-list = ("Loong Kuan Lee", "Ragavi Krishnamoorthy", "Nico Piatkowski")
#let authors = author-list.join(", ")
#let authors-lead = author-list.enumerate().map(((i, a)) => {
    if i == 0 { strong(a) } else { a }
}).join(", ")
#let institutes = "Fraunhofer IAIS · Germany"
#pop.title-box(
    headline,
    // Betterposters: the header band carries ONLY the message, set as large as
    // it will go (title-size 100pt, above) so it reads across the hall. The
    // formal title and presenting author go in the strip immediately below —
    // NOT in the footer. Reason: on A0 portrait a footer sits at ~waist height
    // and is behind bodies all session, which is exactly the identity info you
    // need when you cannot get close (approach / programme-match / photograph).
    // This is also where Morrison's own official portrait template puts them.
    // authors: authors,
    // institutes: institutes,
    logo: tiaoma.qrcode(qr-url, width: 100%, options: (
        bg-color: white,
        whitespace-width: qr-spacing,
        whitespace-height: qr-spacing)),
)

// ---- masthead: all identity in one strip, above head height -----------------
// Replaces the old bottom footer, which cost 5.2cm (4.3% of the sheet) to carry
// an institute line plus the logo. Consolidating here frees that space, drops a
// structural element, and removes the hazard that a `place`d footer reserves no
// area and can be silently overlapped by a growing column.
// NB the two grid rows are fixed heights (4.75em / 1em): a formal title that
// wraps to three lines would clip, so re-check this strip if the title changes.
#v(-0.4em)
#block(width: 100%, inset: (x: 0.6em, top: 1em, bottom: 0.25em),
    // stroke: (bottom: 3pt + fhg-green),
    grid(
        rows: (4.75em, 1em),
        row-gutter: 0.25em,
        align: top,
        grid(
            columns: (1fr, auto),
            column-gutter: 2em,
            align: top,
            [
                #text(size: 1.75em, weight: "regular")[#formal-title]
            ],
            // image("iais_85mm_rgb.png", width: 8.5cm),
            image("iais_85mm_rgb.png", height: 3.50em),
        ),    
        text(size: 1.35em, fill: fhg-grey.darken(45%))[
            // presenting author bolded: attendees need to know who is here
            #grid(
            columns: (1fr, auto), authors-lead, institutes
            )
        ]
    )
)



// ===========================================================================
//  SUMMARY BOX  (the "billboard" element: the 30-second version)
// ===========================================================================
// Full width, so it sits above the two-column body. It absorbs what used to be
// the left column's "Motivation" box and the right column's "Takeaways" box —
// keeping those as well would say the same things twice. The detail sections
// below now start straight at "What is Faithfulness?" / "Contribution".
#block(
    width: 100%,
    fill: fhg-green.lighten(90%),
    // stroke: (left: 8pt + fhg-green, right: 8pt + fhg-green),
    // stroke: (left: 8pt + fhg-green),
    inset: (x: 1em, y: 1.5em),
    grid(
        columns: (1fr, 1fr),
        column-gutter: 1.6em,
        align: top,

        [
            #text(fill: fhg-blue, weight: "bold", size: 1.05em)[Our Motivation]
            #v(0em)
            - *Markov blanket discovery* finds the minimal set shielding a target
              $Y$. Underpins Bayesian / Markov network structure learning.

            - Nearly every method assumes *faithfulness*: an independence in
              the distribution $P$ implies a separation in the graph $G$.

            - But faithfulness can be *violated*, for example: on XOR/parity
              relations, and on finite samples
        ],

        [
            #text(fill: fhg-blue, weight: "bold", size: 1.05em)[Our Contributions]
            #v(0em)
            - *$k$-order faithfulness*: assume an edge must still reveal itself in the
              distribution, but up to $k$ *witness* variables may be needed to see it.

            - *kOMB*: A proof of concept Grow-and-Shrink type
              algorithm. Provably recovers the blanket under $k$-order
              faithfulness.

            - *Result*: kOMB $k = 1$ beats *all eight* baselines on *all
              four* benchmarks. On parity every baseline scores $0.03$
              where kOMB reaches #hl[1.00].
            
        ],
    ),
)

// ===========================================================================
//  BODY
// ===========================================================================
#columns(2, [

    // ------------------------------------------------ LEFT COLUMN
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


    #pop.column-box(heading: "When Faithfulness is Violated")[
        // Intro text beside a single shared DAG: both examples live on the same
        // collider, so the graph is hoisted out instead of redrawn per panel.
        There are two ways the faithfulness assumption can be violated:
        #grid(
            columns: (1fr, auto),
            column-gutter: -1em,
            align: horizon,
            inset: (left: 1em, right: 1.5em),
            [
                - *Empirical* --- faithful $P$ and $G$, but unlucky sample; more data fixes it.

                - *Structural* --- $P$ itself is unfaithful to $G$; no sample size helps.
            ],

            // the one graph both examples share
            [
                #set align(center)
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
                #v(-0.5em)
                #text(size: 0.7em)[same $G$ in both]
            ],
        )

        #v(0.3em)

        // ---- the two violations, side by side ----
        // Each panel puts the evidence beside a VERTICAL implication, so the
        // pair fits half a column. Order is still P above G, and the arrow
        // still points at the conclusion (P) as in the horizontal version.
        // The negated up-arrow comes from `nimplies-up` (a rotated glyph).
        #let break-panel(caption-body, tab-title, tab, p-side, g-side, note) = [
            #figure(
                figure-block[
                    #v(0.25em)
                    #text(size: 0.8em, fill: black)[#tab-title]
                    #v(-0.5em)
                    #align(center, grid(
                        columns: (auto, auto),
                        column-gutter: 0.7em,
                        align: horizon,

                        // the logic table / sample, named directly above it
                        grid.cell(align: horizon + center)[
                            #tab
                        ],

                        // the broken implication, read top-down (P above G, as in fig1)
                        grid.cell(align: horizon + center)[
                            #set align(center)
                            #p-side
                            // #grid(
                            //     columns: (auto, auto),
                            //     column-gutter: 0.25em,
                            //     align: horizon,
                            //     text(size: 1.6em, fill: alert, weight: "bold")[
                            //         #nimplies-down
                            //     ]
                            // )
                            #v(-1.5em)
                            #text(size: 1.6em, fill: alert, weight: "bold")[
                                #nimplies-down
                            ]
                            #v(-1.5em)
                            #g-side
                        ],
                    ))
                    #v(-0.4em)
                    #align(center, text(size: 0.72em)[#note])
                    #v(0.5em)

                ],
                numbering: none,
                caption: figure.caption(caption-body, position: top),
            )
        ]

        #let tab-style(..rows) = table(
            columns: 3,
            align: center,
            inset: (x: 0.5em, y: 0.25em),
            stroke: none,
            table.hline(stroke: 2.5pt),
            table.header([$X$], [$Z$], [$Y$]),
            table.hline(stroke: 1pt),
            ..rows.pos(),
            table.hline(stroke: 2.5pt),
        )

        #grid(
            columns: (1fr, 1fr),
            column-gutter: 0.8em,

            // ---------- (1) empirical: the same faithful AND gate ----------
            // 4-row sample, one noise flip (red), gives EXACT empirical
            // independence between Y and X while Z stays dependent.
            break-panel(
                [*Empirical*],
                [Noisy sample from *AND*],
                tab-style(
                    [0], [0], [0],
                    [0], [1], text(fill: alert, weight: "bold")[1],
                    [1], [0], [0],
                    [1], [1], [1],
                ),
                [
                    #text(size: 0.85em)[$Y #ci($hat(P)$) X$]
                    // #v(0.15em)
                    // #text(size: 0.58em)[$hat(P)(X,Y) = hat(P)(X) hat(P)(Y)$]
                ],
                text(size: 0.85em)[$Y #nci($G$) X$],
                [One noisy flip makes $Y$ *appear* independent of $X$],
            ),

            // ---------- (2) structural: noisy XOR ----------
            break-panel(
                [*Structural*],
                [Full samples from *XOR*],
                tab-style(
                    [0], [0], [0],
                    [0], [1], [1],
                    [1], [0], [1],
                    [1], [1], [0],
                ),
                [
                    #text(size: 0.82em)[$Y #ci($P$) X$]
                    // #text(size: 0.82em)[$Y #ci($P$) Z$]
                ],
                [
                    #text(size: 0.82em)[$Y #nci($G$) X$]
                    // #text(size: 0.82em)[$Y #nci($G$) Z$]
                ],
                [Visible only as $Y #nci($P$) X mid(|) Z$\ needs order $k = 1$.],
            ),
        )
    ]
    
    // ------------------------------------------------ RIGHT COLUMN

    #colbreak()


    #pop.column-box(heading: [$k$-Order Faithfulness])[
        Edge might need up to $k$ extra variables to see it in $P$.

        // The ladder replaces the formula panel and the prose bullets: one
        // column per order, same tested edge (green), growing witness set
        // (orange, W — the paper's Z, renamed to avoid the example variables).
        // Row 3 shows the edge invisible below its order, row 4 the witness
        // set that reveals it, row 5 which assumption covers that rung.
        #let rung(n-par) = {
            let R = 0.85em
            let xs = range(n-par).map(i => i * 1.0)
            let cx = (xs.at(0) + xs.at(-1)) / 2
            diagram(
                node-stroke: 1pt + black,
                node-fill: white,
                spacing: (2.1em, 1.5em),
                // tested variable X — the green edge into Y is the one to detect
                node((0, 0), $X$, radius: R, stroke: 1.6pt + fhg-green),
                edge((0, 0), (cx, 1), "-|>", stroke: 1.6pt + fhg-green),
                // witnesses
                ..range(1, n-par).map(i => node((xs.at(i), 0),
                    text(fill: fhg-orange)[$W_#i$],
                    radius: R, stroke: 1.6pt + fhg-orange,
                    )),
                ..range(1, n-par).map(i => edge((xs.at(i), 0), (cx, 1), "-|>")),
                node((cx, 1), $Y$, radius: R),
            )
        }
        #let dim = fhg-grey.darken(30%)

        #let fig4 = figure-block[
            #align(center, grid(
                columns: (auto, auto, auto),
                row-gutter: 0.55em,
                column-gutter: 1.6em,
                align: center + horizon,

                // ---- row 1: the order ----
                text(weight: "bold", size: 0.85em)[$k = 0$],
                text(weight: "bold", size: 0.85em)[$k = 1$ #text(size: 0.75em)[(XOR)]],
                text(weight: "bold", size: 0.85em)[$k = 2$ #text(size: 0.75em)[(parity)]],

                // ---- row 2: the DAGs ----
                rung(1), rung(2), rung(3),

                // ---- row 3: testing the pair alone ----
                [#text(size: 0.78em)[$Y #nci($P$) X$]
                 #text(size: 0.7em, fill: fhg-green)[$checkmark$]],
                [#text(size: 0.78em, fill: dim)[$Y #ci($P$) X$]
                 #text(size: 0.7em, fill: alert)[$crossmark$ invisible]],
                [#text(size: 0.78em, fill: dim)[$Y #ci($P$) X mid(|) W_1$]
                 #text(size: 0.7em, fill: alert)[$crossmark$ invisible]],

                // ---- row 4: with the witness set ----
                text(size: 0.7em)[no witness needed],
                [#text(size: 0.78em)[$Y #nci($P$) X mid(|) #text(fill: fhg-orange)[$W_1$]$]
                 #text(size: 0.7em, fill: fhg-green)[$checkmark$]],
                [#text(size: 0.78em)[$Y #nci($P$) X mid(|) #text(fill: fhg-orange)[${W_1, W_2}$]$]
                 #text(size: 0.7em, fill: fhg-green)[$checkmark$]],

                // ---- row 5: which assumption covers this rung ----
                text(size: 0.66em)[standard faithfulness],
                text(size: 0.66em)[2-adjacency],
                text(size: 0.66em)[
                    $k$-order faithfulness (ours)
                ],
            ))
        ]

        #figure(
            fig4,
            numbering: none,
            caption: figure.caption([
                Increasing order admits edges invisible on lower
                orders. Generally, parity over $k + 2$ variables needs
                order $k$.
            ], position: bottom)
        )
    ]

    #pop.column-box(heading: "Results")[
        *kOMB* ($k=1$) beats *every* baseline on *all four* benchmarks.

        // ---- benchmark dot plot ------------------------------------------
        // Replaces the old benchmark table. All 8 baselines as grey ticks
        // (best whiskered), kOMB k=1 filled / k=2 hollow orange with ±1 SE
        // whiskers, lanes grouped by cardinality. Data from
        // experiments/output/real_f1.csv; SE = std over the 10 targets /sqrt(10).
        #let bench-data = (
            // `card` = mean domain size |X| per variable, measured over all ten
            // s5000 samples in experiments/data/<net>/. Lanes are ordered by it,
            // so the "higher cardinality favours lower k" trend is spatial.
            (name: "Alarm1",    card: "2.8", bl: (0.674, 0.343, 0.764, 0.693, 0.681, 0.769, 0.737, 0.660), best: 5, bestse: 0.059, k1: (0.780, 0.036), k2: (0.821, 0.061)),
            (name: "Insurance", card: "3.3", bl: (0.659, 0.461, 0.675, 0.629, 0.628, 0.698, 0.630, 0.565), best: 5, bestse: 0.044, k1: (0.714, 0.042), k2: (0.734, 0.051)),
            (name: "Barley",    card: "8.7", bl: (0.339, 0.211, 0.340, 0.337, 0.337, 0.330, 0.228, 0.215), best: 2, bestse: 0.041, k1: (0.516, 0.062), k2: (0.313, 0.050)),
            (name: "Mildew",    card: "15.4", bl: (0.495, 0.287, 0.492, 0.468, 0.467, 0.486, 0.416, 0.273), best: 0, bestse: 0.034, k1: (0.622, 0.050), k2: (0.436, 0.044)),
        )
        #let grey-dark = rgb("#6E7676")
        #let bench-chart = cetz.canvas(length: 1cm, {
            import cetz.draw: *
            let W = 23.0
            let LM = 7.7
            let lane-h = 2.3     // roomier: value labels sit inside the lane
            let x0 = 0.15
            let x1 = 0.90
            let fx(v) = LM + (v - x0) / (x1 - x0) * W

            // SE whisker with end caps, so it reads as an interval
            let whisker(xa, xb, y, col) = {
                line((xa, y), (xb, y), stroke: 2.6pt + col)
                line((xa, y - 0.14), (xa, y + 0.14), stroke: 2.6pt + col)
                line((xb, y - 0.14), (xb, y + 0.14), stroke: 2.6pt + col)
            }

            // ---- lanes, grouped by cardinality with horizontal headers ----
            // (legend lives outside the canvas, in a measured typst grid)
            let y = -0.35
            for (i, d) in bench-data.enumerate() {
                y -= lane-h / 2 + 0.35
                let y1 = y + 0.5     // k = 1 tier
                let y2 = y - 0.5     // k = 2 tier

                content((LM - 0.5, y), anchor: "east", box[
                    // Plain words, not |X|: the paper uses X for a SET of variables,
                    // so |X| would read as a variable count, not a domain size.
                    #text(size: 0.62em, fill: grey-dark)[($approx$#d.card states)]
                    #h(0.3em)
                    #text(size: 0.85em, weight: "bold")[#d.name]
                ])

                // range of all 8 baselines: one soft band instead of 8 ticks
                let lo = calc.min(..d.bl)
                let hi = calc.max(..d.bl)
                rect((fx(lo), y - 0.24), (fx(hi), y + 0.24),
                    fill: fhg-grey.lighten(55%), stroke: none)

                // best baseline: dark dot + SE whisker on the centreline
                let bb = d.bl.at(d.best)
                whisker(fx(bb - d.bestse), fx(bb + d.bestse), y, grey-dark)
                // square, not a circle: shape (not hue) has to carry this, or the
                // baseline and kOMB marks merge in greyscale (contrast 1.46:1)
                rect((fx(bb) - 0.2, y - 0.2), (fx(bb) + 0.2, y + 0.2),
                    fill: grey-dark, stroke: none)

                // kOMB k = 1 (filled) above, k = 2 (hollow) below
                let (m1, s1) = d.k1
                whisker(fx(m1 - s1), fx(m1 + s1), y1, fhg-orange)
                circle((fx(m1), y1), radius: 0.24, fill: fhg-orange, stroke: none)
                content((fx(m1), y1 + 0.36), anchor: "south",
                    text(size: 0.62em, fill: fhg-orange, weight: "bold")[
                        #calc.round(m1, digits: 2)
                    ])
                let (m2, s2) = d.k2
                whisker(fx(m2 - s2), fx(m2 + s2), y2, fhg-orange)
                circle((fx(m2), y2), radius: 0.24, fill: white, stroke: 3pt + fhg-orange)
                content((fx(m2), y2 - 0.36), anchor: "north",
                    text(size: 0.62em, fill: fhg-orange)[#calc.round(m2, digits: 2)])

                y -= lane-h / 2
            }

            // ---- x axis ----
            let ax = y - 0.5
            line((LM, ax), (LM + W, ax), stroke: 1.2pt + black)
            for t in (0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8) {
                line((fx(t), ax), (fx(t), ax - 0.18), stroke: 1.2pt + black)
                content((fx(t), ax - 0.30), anchor: "north", text(size: 0.65em)[#t])
            }
            content((LM + W + 0.3, ax - 0.05), anchor: "west",
                text(size: 0.7em, weight: "bold")[F1])
        })

        // Legend as a measured grid: auto columns cannot overlap, unlike
        // hand-placed canvas labels. Swatches are tiny cetz canvases.
        #let sw(body) = cetz.canvas(length: 1cm, body)
        #let legend = grid(
            columns: 9,
            column-gutter: (0.35em, 1.1em) * 4 + (0.35em,),
            align: horizon,
            sw({
                import cetz.draw: *
                rect((0, -0.2), (1.0, 0.2), fill: fhg-grey.lighten(55%), stroke: none)
            }),
            text(size: 0.66em, fill: grey-dark)[8 baselines],
            sw({
                import cetz.draw: *
                line((0, 0), (1.0, 0), stroke: 2.6pt + grey-dark)
                line((0, -0.14), (0, 0.14), stroke: 2.6pt + grey-dark)
                line((1.0, -0.14), (1.0, 0.14), stroke: 2.6pt + grey-dark)
                rect((0.3, -0.2), (0.7, 0.2), fill: grey-dark, stroke: none)
            }),
            text(size: 0.66em, fill: grey-dark)[their best],
            sw({
                import cetz.draw: *
                circle((0.25, 0), radius: 0.24, fill: fhg-orange, stroke: none)
            }),
            text(size: 0.66em, fill: fhg-orange, weight: "bold")[kOMB $k = 1$],
            sw({
                import cetz.draw: *
                circle((0.25, 0), radius: 0.24, fill: white, stroke: 3pt + fhg-orange)
            }),
            text(size: 0.66em, fill: fhg-orange, weight: "bold")[$k = 2$],
            text(size: 0.58em, fill: grey-dark)[#h(0.6em) bars: $plus.minus 1$ SE],
        )

        - *Recovers Parity Perfectly.* On synthetic parity datasets,
          every baseline scores $0.03$ while kOMB with $k=2$ reaches #hl[1.00].

        #figure(
            figure-block[
                #align(center, legend)
                // Negative: the legend block and the chart canvas each carry
                // their own leading/descender space, which left a visible band
                // between them. (NB `stack(spacing: ...)` clamps negatives —
                // only an explicit #v() in the content flow pulls them together.)
                #v(-0.7em)
                #align(center, bench-chart)
            ],
            numbering: none,
            caption: figure.caption([
                #text(size: 0.85em)[Mean F1 on benchmark networks, $N = 5000$.]
            ], position: top),
        )

        // Why the hollow marks matter: k=2 > k=1 on the low-cardinality nets is
        // the evidence that real networks carry order-2 dependence — the case
        // beyond 2-adjacency faithfulness, i.e. the reason this paper exists.
        - *A bigger search budget cuts both ways*: it wins on Alarm1 and
          Insurance, but on
          high-cardinality nets its CI tests lose power, and $k=1$ wins.

        // Cost stated against the real baseline envelope. NB an earlier draft
        // said "baselines run in <=6 s" — false: LRH needs 89.9 s on Alarm1 and
        // 45.4 s on Insurance (runtime.csv), which kOMB k=1 actually beats.
        - *The cost.* Runtime grows with the budget: $k=1$ takes $5$--$58$ s,
          $k=2$ takes $14$--$336$ s, $k=3$ hits a 30-min cap (F1 $0.00$).
    ]


    // Nothing is cited yet, so this renders as a heading over an empty box and
    // just costs column height. Re-enable once the sections carry citations.
    // #pop.column-box(heading: "References")[
    //     #set text(size: 0.62em)
    //     #bibliography("bibliography.bib", title: none, style: "ieee")
    // ]


])
