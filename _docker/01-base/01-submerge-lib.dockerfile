FROM rust:1.88-bookworm AS builder
RUN mkdir -p /submerge/bin \
    && mkdir -p /submerge/src
WORKDIR /submerge/src
COPY ./. ./
# add required nightly WASM target
RUN rustup component add rust-src \
    && rustup target add wasm32-unknown-unknown \
    && rustup default nightly \
    && rustup target add wasm32-unknown-unknown \
    && rustup default stable
# build SubVT backend
RUN cargo build --release
# copy executables
RUN cp target/release/submerge /submerge/bin/

FROM debian:bookworm-slim
# make bin directory
RUN mkdir -p /submerge/bin
# copy executables
COPY --from=builder /submerge/bin/ /submerge/bin/