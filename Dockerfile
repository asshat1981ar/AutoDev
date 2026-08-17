FROM rust:bookworm AS builder
WORKDIR /src

COPY crates ./crates
RUN cd crates && cargo build --release -p autodev-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home --home-dir /home/autodev autodev

COPY --from=builder /src/crates/target/release/autodev-server /usr/local/bin/autodev-server

ENV AUTODEV_PORT=8080
EXPOSE 8080
USER autodev
ENTRYPOINT ["/usr/local/bin/autodev-server"]
