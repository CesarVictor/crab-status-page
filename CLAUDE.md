# Crab Status Page 🦀

## What this project is

A monitoring/uptime service with a Ferris the crab themed dashboard.
Users add URLs to monitor, the backend pings them on an interval,
and the dashboard shows live status with animated Ferris crabs
(happy = up, sad = down). Includes uptime history, incidents timeline,
and response time graphs.

Built as both a school assignment (Cloud Run deployment) and a
portfolio project.

## Owner profile

- 4th year CS student
- Rust: solid (Axum + JWT + middleware experience, FFI/unsafe touched)
- Docker: experienced (has deployed to cloud before)
- Frontend: React/Next.js (but this project uses a lightweight frontend)
- Goal: impress school + show on portfolio

## School assignment requirements

- [x] Deploy on Google Cloud Run
- [x] Dockerfile that builds and runs locally
- [x] docker-compose.yml for local dev
- [x] Push image to GCP Container Registry
- [x] Deploy to Cloud Run
- [x] Automated deployment via GitHub integration with Cloud Run
- [x] Database required
- [x] Any language (we chose Rust)

## Architecture

```
┌──────────┐     ┌──────────────┐     ┌────────────────┐
│  Browser  │────▶│  Cloud Run   │────▶│  Neon Postgres │
│(Dashboard)│◀────│  (Axum API)  │◀────│  (free tier)   │
└──────────┘     └──────┬───────┘     └────────────────┘
                        │
                        │ Background task
                        ▼
                 ┌──────────────┐
                 │  HTTP pings  │
                 │  to monitored│
                 │  URLs        │
                 └──────────────┘
```

### Components

- **Backend**: Rust + Axum (REST API + background health checker)
- **Database**: PostgreSQL on Neon (free tier, no cost)
- **Frontend**: Static HTML/CSS/JS served by Axum (single binary deployment)
  - No React/Next.js — keep it simple, lightweight, deployable as one container
  - Uses vanilla JS + Chart.js for graphs + CSS animations for Ferris
- **Container**: Docker multi-stage build (builder + runtime)
- **CI/CD**: GitHub Actions → build image → push to GCR → deploy to Cloud Run
- **Registry**: Google Container Registry (gcr.io) or Artifact Registry

### Why this architecture

- **Single container**: Axum serves both API and static frontend.
  Cloud Run charges per request — one service = simple + cheap.
- **Neon PostgreSQL**: free tier, managed, no setup on GCP needed.
  Avoids Cloud SQL ($$$) and SQLite (not great for Cloud Run's
  ephemeral filesystem).
- **No ORM**: SQLx with raw SQL queries. Compile-time checked queries,
  no Diesel complexity.
- **Background health checks**: tokio::spawn a loop that pings
  monitored URLs at their configured interval. On Cloud Run, this
  runs while the container is active (min-instances=1 recommended,
  or use Cloud Scheduler to keep it warm).

## Tech stack

- **Language**: Rust (stable)
- **Framework**: Axum 0.7+
- **Database**: PostgreSQL (Neon free tier)
- **DB layer**: SQLx (compile-time checked queries, async)
- **Async runtime**: Tokio
- **HTTP client**: reqwest (for health checks)
- **Serialization**: serde + serde_json
- **Auth**: None for v1 (public dashboard). Optional API key for
  write endpoints if time permits.
- **Frontend**: Vanilla HTML/CSS/JS + Chart.js
- **Container**: Docker multi-stage build
- **CI/CD**: GitHub Actions
- **Deployment**: Google Cloud Run
- **Registry**: Google Artifact Registry

## Database schema

```sql
-- Services to monitor
CREATE TABLE services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    check_interval_seconds INT NOT NULL DEFAULT 60,
    expected_status_code INT NOT NULL DEFAULT 200,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Individual health check results
CREATE TABLE health_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    status VARCHAR(10) NOT NULL, -- 'up' or 'down'
    response_time_ms INT, -- null if timeout/error
    status_code INT, -- null if connection failed
    error_message TEXT, -- null if successful
    checked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Incidents (when a service goes down)
CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ, -- null if ongoing
    cause TEXT
);
```

## API endpoints

```
GET    /api/services              — list all monitored services
POST   /api/services              — add a service to monitor
GET    /api/services/:id          — get service details + recent checks
PUT    /api/services/:id          — update service config
DELETE /api/services/:id          — remove a service

GET    /api/services/:id/checks   — paginated health check history
GET    /api/services/:id/uptime   — uptime percentage (24h, 7d, 30d)

GET    /api/incidents             — list all incidents
GET    /api/incidents/active      — list ongoing incidents

GET    /api/stats                 — global stats (total services,
                                    avg uptime, total checks today)

GET    /                          — serves the dashboard (static HTML)
GET    /assets/*                  — serves static files (JS, CSS, images)
```

## Project structure

```
crab-status-page/
├── src/
│   ├── main.rs                 # Entry point: start server + bg tasks
│   ├── config.rs               # Env vars, database URL, port
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs             # SQLx pool setup
│   │   └── migrations/         # SQL migration files
│   ├── models/
│   │   ├── mod.rs
│   │   ├── service.rs          # Service struct + queries
│   │   ├── health_check.rs     # HealthCheck struct + queries
│   │   └── incident.rs         # Incident struct + queries
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── services.rs         # CRUD handlers
│   │   ├── checks.rs           # Health check history
│   │   ├── incidents.rs        # Incident endpoints
│   │   └── stats.rs            # Global stats
│   ├── routes.rs               # Router setup
│   ├── monitor/
│   │   ├── mod.rs
│   │   └── checker.rs          # Background health check loop
│   └── error.rs                # AppError type, into_response
├── static/                     # Frontend (served by Axum)
│   ├── index.html
│   ├── css/
│   │   └── style.css           # Ferris animations, dashboard layout
│   ├── js/
│   │   ├── app.js              # Dashboard logic
│   │   ├── charts.js           # Chart.js graphs
│   │   └── ferris.js           # Crab animations
│   └── img/
│       ├── ferris-happy.svg
│       ├── ferris-sad.svg
│       └── ferris-checking.svg
├── migrations/
│   └── 001_initial.sql
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
├── .github/
│   └── workflows/
│       └── deploy.yml          # Build → push → Cloud Run
├── .env.example
├── Cargo.toml
├── CLAUDE.md
└── README.md
```

## Docker strategy

### Multi-stage Dockerfile

```dockerfile
# Stage 1: Build
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cache deps layer
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/crab-status-page /usr/local/bin/
COPY --from=builder /app/static /static
COPY --from=builder /app/migrations /migrations
ENV PORT=8080
EXPOSE 8080
CMD ["crab-status-page"]
```

### docker-compose.yml (local dev)

Runs the API + a local PostgreSQL. The app connects to the local
DB instead of Neon.

## CI/CD pipeline (GitHub Actions)

```
Push to main
    │
    ▼
GitHub Actions
    ├── cargo test
    ├── cargo clippy
    ├── docker build
    ├── docker push → Artifact Registry
    └── gcloud run deploy → Cloud Run
```

## Cloud Run specifics

- **Port**: 8080 (Cloud Run default)
- **Min instances**: 1 (keeps the health checker running; costs ~$0
  on free tier if low traffic)
- **Max instances**: 1 (school project, no need to scale)
- **CPU allocation**: "always on" (needed for background health checks)
- **Memory**: 256MB (Rust is tiny)
- **Env vars**: DATABASE_URL, RUST_LOG

## Frontend dashboard concept

The dashboard has:
1. **Header**: "Crab Status Page 🦀" with Ferris waving
2. **Services grid**: cards per service, each with:
   - Ferris happy (green) or sad (red) animation
   - Service name + URL
   - Current status + response time
   - Uptime bar (last 30 days, like GitHub status page)
3. **Incidents timeline**: chronological list of downtime events
4. **Stats bar**: total services, avg uptime %, checks today
5. **Add service form**: URL + name + interval

Visual style: playful but clean. Dark background, neon green/red
accents, pixel-ish font for headers, smooth CSS animations on Ferris.

## Code conventions

- No `any` equivalent — use proper error types
- All queries via SQLx macros (`sqlx::query_as!`) for compile-time check
- Error handling: thiserror for custom errors, `?` propagation
- Handlers return `Result<Json<T>, AppError>`
- AppError implements IntoResponse (proper HTTP status codes)
- Env config via dotenvy + std::env (no complex config crate)
- Log via tracing + tracing-subscriber

## Commands

- `cargo run` — run locally (needs DATABASE_URL)
- `cargo test` — run tests
- `cargo clippy` — lint
- `docker compose up` — run with local PostgreSQL
- `docker build -f docker/Dockerfile -t crab-status-page .` — build image
- `docker run -p 8080:8080 -e DATABASE_URL=... crab-status-page` — run image

## Working preferences with Claude Code

- Propose a plan before coding any new feature
- Implement file by file, pause for review on complex parts
- Generate conventional commit messages automatically, don't ask
- Never mention Claude, AI, or that code was AI-generated in any
  commit message, PR description, comment, or documentation
- Keep this file in sync. Update "Current status" at end of session.

## Current status

**Not started** — CLAUDE.md created, ready for implementation.

## Roadmap

1. **Phase 1 — Core API**: Axum server, DB pool, service CRUD,
   health check model + queries, error handling
2. **Phase 2 — Health checker**: Background task that pings services,
   writes results to DB, creates/resolves incidents
3. **Phase 3 — Dashboard**: Static HTML/CSS/JS frontend, Ferris
   animations, Chart.js graphs, auto-refresh
4. **Phase 4 — Docker**: Dockerfile, docker-compose.yml, local testing
5. **Phase 5 — CI/CD + Deploy**: GitHub Actions workflow, Artifact
   Registry, Cloud Run deployment, env vars
6. **Phase 6 — Polish**: README, error handling edge cases, cleanup