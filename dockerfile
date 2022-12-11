FROM rust:slim-buster as builder
RUN apt-get update && apt-get install -y librust-openssl-sys-dev
WORKDIR /usr/src/configserver
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/configserver /usr/local/bin/configserver
CMD ["configserver"]