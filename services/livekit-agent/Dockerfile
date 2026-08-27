FROM node:24-bookworm-slim AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY services/livekit-agent/package.json services/livekit-agent/package.json
RUN pnpm install --frozen-lockfile --filter @autodev/livekit-agent...
COPY services/livekit-agent services/livekit-agent
RUN pnpm --filter @autodev/livekit-agent build \
    && pnpm --filter @autodev/livekit-agent deploy --legacy --prod /deploy \
    && rm -rf /deploy/src /deploy/test /deploy/dist/test \
    && find /deploy/dist -type f \( -name '*.map' -o -name '*.d.ts' \) -delete

FROM node:24-bookworm-slim AS runtime
WORKDIR /app
RUN useradd --system --uid 10001 --create-home --home-dir /home/autodev autodev
COPY --from=build --chown=autodev:autodev /deploy ./
USER autodev
EXPOSE 8081
ENTRYPOINT ["node", "dist/src/main.js"]
CMD ["start"]