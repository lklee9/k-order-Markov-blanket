// Deliberate no-op that SHADOWS the theme's own assets/js/scale.fix.js.
//
// jekyll-theme-minimal 0.2.0 unconditionally loads this path from its layout,
// and the gem's version rewrites the viewport meta to
//     width=device-width, minimum-scale=1.0, maximum-scale=1.0
// on any UA matching /iPhone/i, widening to 0.25-1.6 only for the duration of a
// touch gesture. That locks pinch-zoom on exactly the devices most people will
// read this page on, which fails WCAG 2.1 SC 1.4.4 (Resize Text) and removes the
// reader's only escape hatch if any figure or table is too small for them.
//
// A file at this path in the site source wins over the gem's copy, so keeping it
// empty simply leaves the layout's `<meta name="viewport" content="width=device
// -width, initial-scale=1">` alone. Do not delete this file to "clean up" — the
// zoom lock comes straight back.
