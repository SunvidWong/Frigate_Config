<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# frigate-config Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-08

## Active Technologies
- Rust 1.75+ (Tauri backend), TypeScript 5.0+ (React/Vue frontend) + Tauri 1.5+, React 18 or Vue 3, shadcn/ui or similar iOS-style component library (001-2-1-ui)

## Project Structure
```
src/
tests/
```

## Commands
cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style
Rust 1.75+ (Tauri backend), TypeScript 5.0+ (React/Vue frontend): Follow standard conventions

## Recent Changes
- 001-2-1-ui: Added Rust 1.75+ (Tauri backend), TypeScript 5.0+ (React/Vue frontend) + Tauri 1.5+, React 18 or Vue 3, shadcn/ui or similar iOS-style component library

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->