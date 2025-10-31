# THIS IS A WORK-IN-PROGRESS / GOALS... NOT FULLY IMPLEMENTED YET!

## Prerequisites

1. The usual stuff (Rust, Docker, NPM, etc.)
2. [Taskfile](https://taskfile.dev/installation)
3. [Install watchexec](https://github.com/watchexec/watchexec?tab=readme-ov-file#install)
4. [Install and configure wkg to pull from wa.dev](https://crates.io/crates/wkg)
5. Copy `.example.env` to `.env` and replace the values
6. Make sure you have `wasm32-wasip2` target installed: `rustup target add wasm32-wasip2`
7. (optional) setup [ngrok](http://ngrok.com) for local server dev and change the url in [config.yml](../taskfile/config.yml)

## TL;DR

Start all backend services
```bash
# Alternatively, if you need more than one operator
# task backend:start-all OPERATORS=3
task backend:start-all
```

```bash
_... do stuff ..._

Stop all backend services
```bash
task backend:stop-all
```

As a helper for local server dev, if you have ngrok set up, you can run run this and get a public url to use for webhooks (it will take up its own terminal, however)

```bash
task backend:ngrok-start
```

## Building

### Contracts

To build the contracts (found in `builds/contracts/`):

```bash
task contracts:build-all
```

Or build a specific contract

```bash
task contracts:build CONTRACT=payments
```

To generate the schema files from the contracts (found in `builds/schema/`)

```bash
task contracts:schema-all
```


### Components

#### First, fetch the wit definitions

This is only needed once, or when the component wits are updated

```bash
task components:fetch-wit-all
```

Or fetch the wit for a specific component

```bash
task components:fetch-wit COMPONENT=operator
```

#### Build the components

```bash
task components:build-all
```

Or build a specific component

```bash
task components:build COMPONENT=operator
```

#### Execute a component directly

This will execute the operator component to read messages from the Telegram bot

```bash
# Read updates that came into the bot
task components:exec-operator-read-updates
# Read those that parse as commands
task components:exec-operator-read-commands
```

## Backend

### Chains

It may take a while for the chain to startup, be patient... chains will be running in the background via docker and do not require their own terminal

Start the chains
```bash
task backend:start-chains
```

Stop the chains
```bash
task backend:stop-chains
```

### Webhook Server

The server runs in the forground and therefore requires its own terminal

Start the server
```bash
task backend:start-server
```

Start the server and watch for changes (auto-restart)
```bash
task backend:start-server-watch
```

### WAVS

Start the operator, aggregator, and telemetry
```bash
# Alternatively, if you need more than one operator
# task backend:start-wavs OPERATORS=3
task backend:start-wavs
```

Stop the operator, aggregator, and telemetry
```bash
task backend:stop-wavs
```

### IPFS

Start a local IPFS server

```bash
task backend:start-ipfs
```

Stop the local IPFS server
```bash
task backend:stop-ipfs
task backend:stop-wavs

### All Backend Services At Once

Start all backend services
```bash
# Alternatively, if you need more than one operator
# task backend:start-all OPERATORS=3
task backend:start-all
```

Stop all backend services
```bash
task backend:stop-all
```

## Testing


### Contracts

*off-chain*

Test a specific contract off-chain

Note that the name is the friendly name set in [config.yml](../taskfile/config.yml) without the `tg-contract` prefix

```bash
task test:contract-off-chain CONTRACT=payments
```

All off-chain tests
```bash
task test:off-chain
```

*on-chain*

Make sure you've [started the chains](#chains) first

```bash
task test:on-chain
# alternatively `task test:contract-on-chain CONTRACT=payments`
```


### Components

Just `cargo test` as needed

### End-to-end services

The flow is similar to on-chain tests, and assumes the contracts are already built

Make sure you've [started the backend services](#all-backend-services-at-once) first

_STATUS: TODO_
```bash
task test:e2e
```

Jaeger UI is at [http://localhost:16686/](http://localhost:16686/)
Prometheus is at [http://localhost:9090/](http://localhost:9090/)

### Multiple operators

_STATUS: TODO_
```bash
# When starting the whole backend
task backend:start-all OPERATORS=N

# If just starting wavs, not chain
task backend:start-wavs-operator OPERATORS=N
```

Make sure you have that number of submission wallets in your `.env`
