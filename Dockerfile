FROM rust
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release
CMD ["cargo", "run", "--release"]
