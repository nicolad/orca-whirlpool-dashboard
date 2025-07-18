# Orca Whirlpool Dashboard

A Rust Actix Web backend with a React frontend.

## Running locally

1. Build the frontend assets:

```bash
cd frontend
pnpm install
pnpm build
```

2. Run the backend:

```bash
cd ..
shuttle run --debug --secrets backend/Secrets.toml
```
