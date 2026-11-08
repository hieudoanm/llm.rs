# AGENTS.md — llm.rs landing page

Guidance for AI agents working on the landing page (`packages/landing`). See the root [`AGENTS.md`](../../AGENTS.md) for repository-wide conventions.

## Overview

The landing page is a plain static site — no framework, no build step, no package manifest, no JavaScript. The whole directory is uploaded verbatim to GitHub Pages at `https://hieudoanm.github.io/llm.rs` (project site, so all asset paths are relative).

## Files

- `index.html` — the entire page (nav, hero + terminal, features, CLI, CTA, footer)
- `styles.css` — the full stylesheet; design tokens live in `:root`
- `404.html` — GitHub Pages 404 page, reuses `styles.css`
- `logo.svg`, `favicon.ico` (generated from `logo.svg`), `robots.txt`, `sitemap.xml`, `humans.txt`
- `AGENTS.md`, `README.md`, `LICENSE` — repo docs; they are uploaded to Pages too, which is harmless

## Preview

There is no build. Serve the directory with any static file server:

```sh
cd packages/landing
python3 -m http.server 4173
```

## Conventions

- **Light theme only.** Colours are CSS custom properties in `styles.css` (`--bg`, `--fg`, `--accent`, …). Do not add a dark theme or theme toggle.
- **No frameworks, no JS, no package.json.** Keep it to HTML + CSS + static assets.
- **Relative asset paths.** The site is served under the `/llm.rs` base path on GitHub Pages; use relative URLs (`logo.svg`, `styles.css`), never absolute `/…` paths.
- **Accent colour is orange** (`--accent: #c2410c` family, `--accent-bright` for text on the dark terminal).
- **Design rules.** Follow the 20 rules of AI unslop design: concrete headlines, one accent colour, consistent icons, no filler badges, real product artefacts over decoration.
- **Docs and downloads** live on GitHub: link to `https://github.com/hieudoanm/llm.rs` (`/releases`, `/tree/master/docs`, `/tree/master/crates/llm-cli`) rather than adding pages.
- **Deploy:** CI uploads `packages/landing` via `actions/upload-pages-artifact` + `actions/deploy-pages` (see `.github/workflows/ci.yaml`). No build step — edit files and they ship as-is.
