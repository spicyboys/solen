FROM rust:1.67 as builder
WORKDIR /usr/src/solen
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/src/solen/target/release/solen /usr/bin/solen
CMD ["solen"]
