# llm.rs landing page

The landing page for llm.rs — a plain static site (HTML + CSS, no framework,
no build step) deployed to GitHub Pages at
<https://hieudoanm.github.io/llm.rs>.

## Files

`index.html`, `styles.css`, `404.html`, and static assets (`logo.svg`,
`favicon.ico`, `robots.txt`, `sitemap.xml`, `humans.txt`) — all at the package
root, all deployed verbatim.

## Preview

```bash
cd packages/landing
python3 -m http.server 4173
```

## Deploy

CI uploads this directory verbatim to GitHub Pages — there is no build step.

Brand assets live in `logo.svg` (source) and `favicon.ico` (generated with
ImageMagick: `magick -background none logo.svg -define icon:auto-resize=16,32,48 favicon.ico`).
Keep them synchronized with the canonical files in `docs/07-ui-ux-design/brand`
and `docs/07-ui-ux-design/og` when branding changes.
