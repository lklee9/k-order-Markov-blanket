// The benchmark F1 dot plot, RE-AUTHORED for a phone rather than re-exported.
//
// Why not just export the poster's canvas: the poster reserves 7.7cm of the
// 30.7cm canvas for right-aligned lane labels, and sets them at 0.62em. Scaled
// down to a 360px-wide phone that puts the labels at roughly 9.5 CSS px —
// unreadable. Here the labels move onto their own full-width line above each
// lane, the axis shortens, the type goes up to 0.8em, and the ticks drop from 7
// to 4, which lands every label at ~19-20 CSS px at 360px.
//
// Same data, same marks, same lane order as the poster. If poster/main.typ's
// `bench-data` ever changes, this file and the results table in index.md both
// have to be updated with it — the numbers live in three places.

#import "common.typ": *
#show: setup

// ---- data: verbatim from poster/main.typ:665-673 ----------------------------
// From experiments/output/real_f1.csv. std there is over 100 runs (10 targets
// x 10 samples); SE = std/sqrt(10), treating the 10 targets as independent.
// `card` = mean states per variable, distinct values POOLED over the ten
// s5000 samples then averaged over variables. Lanes are ordered by it, so the
// "higher cardinality favours lower k" trend is spatial.
#let bench-data = (
    (name: "Alarm1",    card: "2.8",  bl: (0.674, 0.343, 0.764, 0.693, 0.681, 0.769, 0.737, 0.660), best: 5, bestse: 0.059, k1: (0.780, 0.036), k2: (0.821, 0.061)),
    (name: "Insurance", card: "3.3",  bl: (0.659, 0.461, 0.675, 0.629, 0.628, 0.698, 0.630, 0.565), best: 5, bestse: 0.044, k1: (0.714, 0.042), k2: (0.734, 0.051)),
    (name: "Barley",    card: "8.6",  bl: (0.339, 0.211, 0.340, 0.337, 0.337, 0.330, 0.228, 0.215), best: 2, bestse: 0.041, k1: (0.516, 0.062), k2: (0.313, 0.050)),
    (name: "Mildew",    card: "15.4", bl: (0.495, 0.287, 0.492, 0.468, 0.467, 0.486, 0.416, 0.273), best: 0, bestse: 0.034, k1: (0.622, 0.050), k2: (0.436, 0.044)),
)

#cetz.canvas(length: 1cm, {
    import cetz.draw: *

    let W  = 15.0      // axis length
    let LM = 0.35      // left margin: labels are above the lane now, not beside it
    let x0 = 0.15
    let x1 = 0.90
    let fx(v) = LM + (v - x0) / (x1 - x0) * W

    // Vertical budget per lane, all of it forced by the value labels. A k = 1
    // label sits 0.28 above its tier (+0.52) and is ~0.72 tall, so it reaches
    // y + 1.52; the lane's own label line is ~1.2 tall and centred at
    // y + label-dy, so it spans y + 1.8 .. y + 3.0. lane-h then has to exceed
    // 3.0 + 1.52 + a gap, or the next lane's name lands on this lane's k = 2
    // label. Shrink any of these three and re-render — they collide silently.
    let lane-h   = 4.9    // lane pitch, incl. the label line above each lane
    let label-dy = 2.4    // lane centre -> centre of its label line
    let val-dy   = 0.28   // marker tier -> value label

    // SE whisker with end caps, so it reads as an interval and not a smear
    let whisker(xa, xb, y, col) = {
        line((xa, y), (xb, y), stroke: 2.6pt + col)
        line((xa, y - 0.14), (xa, y + 0.14), stroke: 2.6pt + col)
        line((xb, y - 0.14), (xb, y + 0.14), stroke: 2.6pt + col)
    }

    let y = 0.0
    for d in bench-data {
        // ---- label line, left-aligned at the plot's left edge ----
        content((LM, y + label-dy), anchor: "west", box[
            // Plain words, not |X|: the paper uses X for a SET of variables, so
            // |X| would read as a variable count rather than a domain size.
            #text(size: 0.85em, weight: "bold")[#d.name]
            #h(0.4em)
            #text(size: 0.72em, fill: grey-dark)[($approx$#d.card states)]
        ])

        let y1 = y + 0.52    // kOMB k = 1 tier, above the centreline
        let y2 = y - 0.52    // kOMB k = 2 tier, below it

        // range of all 8 baselines: one soft band rather than 8 separate ticks
        rect((fx(calc.min(..d.bl)), y - 0.26), (fx(calc.max(..d.bl)), y + 0.26),
            fill: fhg-grey.lighten(55%), stroke: none)

        // best baseline: a SQUARE, not a circle — shape has to carry this, or it
        // merges with the kOMB marks in greyscale (orange vs grey is 1.46:1)
        let bb = d.bl.at(d.best)
        whisker(fx(bb - d.bestse), fx(bb + d.bestse), y, grey-dark)
        rect((fx(bb) - 0.21, y - 0.21), (fx(bb) + 0.21, y + 0.21),
            fill: grey-dark, stroke: none)

        // kOMB k = 1: filled circle, value label above
        let (m1, s1) = d.k1
        whisker(fx(m1 - s1), fx(m1 + s1), y1, fhg-orange)
        circle((fx(m1), y1), radius: 0.25, fill: fhg-orange, stroke: none)
        content((fx(m1), y1 + val-dy), anchor: "south",
            text(size: 0.72em, fill: fhg-orange, weight: "bold")[
                #calc.round(m1, digits: 2)
            ])

        // kOMB k = 2: hollow circle, value label below
        let (m2, s2) = d.k2
        whisker(fx(m2 - s2), fx(m2 + s2), y2, fhg-orange)
        circle((fx(m2), y2), radius: 0.25, fill: white, stroke: 3pt + fhg-orange)
        content((fx(m2), y2 - val-dy), anchor: "north",
            text(size: 0.72em, fill: fhg-orange)[#calc.round(m2, digits: 2)])

        y -= lane-h
    }

    // ---- x axis: 4 ticks instead of the poster's 7 ----
    // Derived from the LAST lane's centre, not from the loop's leftover `y`,
    // which sits one full pitch below the final lane.
    let ax = -(bench-data.len() - 1) * lane-h - 2.4
    line((LM, ax), (LM + W, ax), stroke: 1.2pt + black)
    for t in (0.2, 0.4, 0.6, 0.8) {
        line((fx(t), ax), (fx(t), ax - 0.18), stroke: 1.2pt + black)
        content((fx(t), ax - 0.32), anchor: "north", text(size: 0.72em)[#t])
    }
    content((LM + W / 2, ax - 1.15), anchor: "north",
        text(size: 0.78em, weight: "bold")[F1])
})
