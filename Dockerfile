FROM rust:1.90-trixie as builder
RUN apt install libopus-dev
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
CMD ["solen"]
