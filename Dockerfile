FROM rust:1.97-trixie AS builder
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release
RUN cargo install topcoat-cli --version 0.5.0 --locked \
    && topcoat asset bundle --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
COPY --from=builder /usr/src/solen/target/assets /usr/bin/assets
CMD ["solen"]
