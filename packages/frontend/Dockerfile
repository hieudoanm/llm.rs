# Build from the repository root so the pnpm workspace lockfile is in context:
#
#   docker build -f packages/frontend/Dockerfile -t skl-frontend .
#
# `NEXT_PUBLIC_API_URL` is inlined at build time, so pass it as a build arg
# (`--build-arg NEXT_PUBLIC_API_URL=http://127.0.0.1:8000`).
# `NEXT_PUBLIC_API_DOCS_URL` is inlined the same way and may be left empty to
# fall back to `${NEXT_PUBLIC_API_URL}/docs`.
# See docs/11-deployment/local-development.md.

FROM node:26.13.0-alpine AS base
ENV PNPM_HOME=/pnpm \
    PATH=/pnpm:$PATH \
    NEXT_TELEMETRY_DISABLED=1 \
    COREPACK_ENABLE_DOWNLOAD_PROMPT=0
WORKDIR /repo

FROM base AS deps
RUN npm install -g --ignore-scripts pnpm@12.14.0
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json ./
COPY packages/frontend/package.json packages/frontend/
COPY packages/backend/package.json packages/backend/
COPY packages/data/package.json packages/data/
RUN pnpm install --frozen-lockfile

FROM base AS builder
RUN npm install -g --ignore-scripts pnpm@12.14.0
COPY . .
ARG NEXT_PUBLIC_API_URL=http://127.0.0.1:8000
ARG NEXT_PUBLIC_API_DOCS_URL=
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_DOCS_URL=$NEXT_PUBLIC_API_DOCS_URL
RUN pnpm --filter @skl/frontend build

FROM base AS runner
ENV NODE_ENV=production
RUN npm install -g --ignore-scripts pnpm@12.14.0
COPY --from=builder /repo/package.json /repo/pnpm-workspace.yaml /repo/pnpm-lock.yaml ./
COPY --from=builder /repo/node_modules ./node_modules
COPY --from=builder /repo/packages/frontend ./packages/frontend
EXPOSE 3000
CMD ["pnpm", "--filter", "@skl/frontend", "exec", "next", "start", "--hostname", "0.0.0.0", "--port", "3000"]
