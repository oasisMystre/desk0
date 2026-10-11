# syntax=docker/dockerfile:1.6
FROM oven/bun:1.4.2 AS base
WORKDIR /usr/src/app

RUN apt-get update && \
  apt-get install -y curl bash ca-certificates gettext-base && \
  rm -rf /var/lib/apt/lists/*

ENV DOCKER_BUILDKIT=1
ENV NODE_ENV="production"
ENV PATH="/root/.bun/bin:$PATH"

FROM base AS prepare

RUN bun add -g turbo@2.10.5
COPY . .
RUN turbo prune @desk0/buildx --docker

FROM base AS builder

COPY --from=prepare /usr/src/app/drizzle.config.ts ./
COPY --from=prepare /usr/src/app/out/json .
RUN bun install

COPY --from=prepare /usr/src/app/out/full .
RUN bun turbo run build

COPY --from=prepare /usr/src/app/drizzle ./drizzle

FROM base AS runner
FROM jkroepke/wixtoolset:latest
WORKDIR /usr/src/app

USER root

COPY --from=base /usr/local/bin/bun /usr/local/bin/bun
COPY --from=builder /usr/src/app ./

RUN cat > /usr/src/app/entrypoint.sh <<'EOF'
#!/bin/sh
bun drizzle:migrate
exec bun run /usr/src/app/workers/buildx/src/index.ts
EOF

RUN chmod +x /usr/src/app/entrypoint.sh
ENTRYPOINT ["/usr/src/app/entrypoint.sh"]