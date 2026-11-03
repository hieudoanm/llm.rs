# llm.rs landing page

The landing page and documentation site for llm.rs (`@llm/landing`). It is a Next.js
static export suitable for GitHub Pages, Netlify, Cloudflare Pages, or any static file
host.

## Development

```bash
pnpm --filter @llm/landing dev
pnpm --filter @llm/landing check
```

## Build

```bash
pnpm --filter @llm/landing build
```

Set `NEXT_PUBLIC_SITE_URL` to the deployed origin when building so Open Graph image
URLs are absolute:

```bash
NEXT_PUBLIC_SITE_URL=https://example.com pnpm --filter @llm/landing build
```

The production output is written to `out/`. The site uses a static export with trailing
slashes and serves documentation at `/docs` from Markdown files in
`packages/landing/docs/`.

Brand assets are stored in `public/images/`. Keep them synchronized with the canonical
files in `docs/07-ui-ux-design/brand` and `docs/07-ui-ux-design/og` when branding
changes.
