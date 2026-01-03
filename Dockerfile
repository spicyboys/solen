FROM rust:1.90-trixie as builder
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN sudo apt-get update && sudo apt --only-upgrade install ca-certificates
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
CMD ["solen"]
