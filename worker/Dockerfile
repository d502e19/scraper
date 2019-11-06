FROM rust:1.38.0-stretch

WORKDIR /usr/src/worker
COPY worker .

RUN apt-get update && apt-get install -y libssl-dev

RUN cargo install --path .

CMD ["worker"]
