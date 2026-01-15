FROM rust:1.91-trixie AS builder
RUN mkdir -p /submerge/bin \
    && mkdir -p /submerge/src
WORKDIR /submerge/src
COPY ./. ./
RUN cd _chainspecs \
    && chmod +x init.sh \
    && ./init.sh \
    && cd ..
RUN apt update && apt install -y clang lld && rm -rf /var/lib/apt/lists/*
# add required WASM targets
RUN rustup component add rust-src \
    && rustup target add wasm32-unknown-unknown \
    && rustup default nightly \
    && rustup target add wasm32-unknown-unknown \
    && rustup default stable
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld -Ccodegen-units=1"
# build Submerge executables
RUN CARGO_BUILD_JOBS=1 cargo build --release
# copy executables
RUN cp target/release/submerge-crystal /submerge/bin/

FROM debian:trixie-slim
# make bin directory
RUN mkdir -p /submerge/bin
# copy executables
COPY --from=builder /submerge/bin/ /submerge/bin/