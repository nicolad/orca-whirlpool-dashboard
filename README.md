# Orca Whirlpool Dashboard

This project consists of a Rust backend and a React frontend. Everything is deployed using [Shuttle](https://www.shuttle.rs/) so there is no AWS, ECS or Terraform in the workflow.

## Prerequisites

```bash
curl -Ls https://www.shuttle.rs/install | bash  # installs `cargo-shuttle`
rustup default stable
corepack enable                                 # installs pnpm
```

## Repository structure

```
.
├─ backend/    # actix-web service (Shuttle binary)
└─ frontend/   # Vite React UI
```

`Shuttle.toml` declares the static assets copied from `frontend/dist` during deploy.

## Building and running locally

1. Build the frontend:

```bash
cd frontend
pnpm install
pnpm build
```

2. Run the backend with Shuttle:

```bash
cd ..
cargo shuttle run -p backend -- --secrets backend/Secrets.toml
```

## Production deploy

Build the frontend and deploy the workspace:

```bash
cd frontend && pnpm install && pnpm build && cd ..
cargo shuttle deploy --workspace
```

Shuttle uploads the backend binary, serves the static files from `frontend/dist` and provides a public URL.

## Continuous deployment

`.github/workflows/ci.yml` can run tests and deploy automatically:

```yaml
- run: pnpm --filter frontend install --frozen-lockfile
- run: pnpm --filter frontend build
- run: cargo test --workspace
- run: cargo shuttle deploy --workspace --no-test --token ${{ secrets.SHUTTLE_TOKEN }}
```

Logs and metrics are available in [console.shuttle.rs](https://console.shuttle.rs/).
