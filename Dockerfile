FROM rust:1.63 as builder
WORKDIR /usr/src/bitcoin-indexer
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/bitcoin-indexer /usr/local/bin/bitcoin-indexer
CMD ["bitcoin-indexer"]
