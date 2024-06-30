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

## Docker

Run the following command to start the server in a Docker container.

```bash
docker-compose up
```

The server will be running on `http://localhost:3000`.

Run the following command to also run the telegram bot.

```bash
docker-compose --profile telegram --profile cloudflare up
```
