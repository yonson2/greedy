FROM rust:1.81-slim as builder

WORKDIR /usr/src/

COPY . .

RUN apt update -y && apt install nodejs npm dav1d -y
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt update -y && apt install curl wget htop vim -y

WORKDIR /usr/app

COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/greedy /usr/app/greedy

ENTRYPOINT ["/usr/app/greedy"]
