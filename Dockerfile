# syntax=docker/dockerfile:1
FROM rust:1.83-slim-bullseye as builder

WORKDIR /sangjeom

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY ./.sqlx ./.sqlx

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/sangjeom/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/sangjeom/target \
    cp /sangjeom/target/release/sangjeom /usr/local/bin/sangjeom

FROM debian:bullseye-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES ffmpeg wait-for-it
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

ENV ROCKET_ADDRESS 0.0.0.0
ENV RUST_LOG debug

COPY --from=builder /usr/local/bin/sangjeom /bin/sangjeom

CMD ["/bin/sangjeom"]
