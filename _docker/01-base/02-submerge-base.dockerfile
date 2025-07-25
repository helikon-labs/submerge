FROM debian:bookworm-slim
# install certificate authority certificates, create config directory
RUN apt update \
    && apt install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && mkdir -p /submerge