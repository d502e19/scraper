FROM rust:1.38.0-stretch

WORKDIR /usr/src/worker
COPY . .

RUN apt update && apt install -y libssl-dev

RUN cargo install --path .

CMD ["worker"]
