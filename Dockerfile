FROM rust:1.90-trixie as builder
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
CMD ["solen"]
