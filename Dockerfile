# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy only manifest files for better caching
COPY Cargo.toml Cargo.lock* ./

# Create empty source file to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the rest of the code
COPY . .

# Force rebuild with our real source code
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/kodem_cards_backend /usr/local/bin/kodem_cards_backend

EXPOSE 3000

CMD ["kodem_cards_backend"]
