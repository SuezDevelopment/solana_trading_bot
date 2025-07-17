FROM rust:latest
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN cargo build --release
CMD ["./target/release/solana-t_bot"]