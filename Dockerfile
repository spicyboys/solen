FROM rust:1.90-trixie as builder
RUN apt-get update && apt-get install -y libopus-dev
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libopus-dev
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
CMD ["solen"]
