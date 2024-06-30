# konnektoren-api

API Server for Konnektoren

## build

```bash
cargo build
```

## run

Copy the `example.env` file to `.env` and set the environment variables.

```bash
RUST_LOG=info cargo run
```

The server will be running on `http://localhost:3000`.

## test

```bash
cargo test
```

## OpenAPI

The OpenAPI documentation is available at `http://localhost:3000/docs/`.
