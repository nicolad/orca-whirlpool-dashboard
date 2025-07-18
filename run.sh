#!/usr/bin/env bash
cd frontend
pnpm build
cd ..
shuttle run --debug --secrets backend/Secrets.toml