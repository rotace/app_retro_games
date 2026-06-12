# Workspace Structure Contract

## Workspace Members
The following crates must be defined in the root `Cargo.toml`:
- `core`
- `runner-debian`
- `runner-wslg`

## Dependency Constraints
- `runner-debian` -> `core` (allowed)
- `runner-wslg` -> `core` (allowed)
- `core` -> `runner-debian` (FORBIDDEN)
- `core` -> `runner-wslg` (FORBIDDEN)

## Entry Points
- `runner-debian`: `src/main.rs`
- `runner-wslg`: `src/main.rs`
- `core`: `src/lib.rs`
