# ── Stage 1: Build ────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /app

# Cache dependency compilation layer separately from app code
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
# Remove dummy artifacts so cargo rebuilds the real binary
RUN rm -f target/release/crab-status-page target/release/deps/crab_status_page*

# Build the real app (deps are already compiled above)
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/crab-status-page /usr/local/bin/crab-status-page
COPY --from=builder /app/static ./static
COPY --from=builder /app/migrations ./migrations

ENV PORT=8080
EXPOSE 8080

CMD ["crab-status-page"]
