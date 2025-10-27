# THIS IS A WORK-IN-PROGRESS / GOALS... NOT FULLY IMPLEMENTED YET!

## Prerequisites

1. The usual stuff (Rust, Docker, NPM, etc.)
2. [Taskfile](https://taskfile.dev/installation)
3. Copy `.example.env` to `.env` and replace the values

## Building

#### Contracts

```bash
task contracts:build-all
```

#### Components

_STATUS: TODO_
```bash
task components:build-all
```

## Testing


### Contracts

*off-chain*

Test a specific contract off-chain
```bash
task test:contract-off-chain CONTRACT=<contract-name>
```

All off-chain tests
```bash
task test:off-chain
```

*on-chain*

This requires first start the chains, running the tests, and then remembering to shut it down

It may take a while for the chain to startup, be patient... recommendation is to leave it up while developing

```bash
task backend:start-chains
 # alternatively `task test:contract-on-chain CONTRACT=<contract-name>`
task test:on-chain
task backend:stop-chains
```


### Components

Just `cargo test` as needed

### End-to-end services

The flow is similar to on-chain tests, and assumes the contracts are already built

_STATUS: TODO_
```bash
task backend:start-all
task test:e2e
task backend:stop-all
```

It may take a while for the backend to startup, recommendation is to leave it up while developing

If you already have the chains running, then run `task backend:start-wavs` instead of `task backend:start-all`

Jaeger UI is at [http://localhost:16686/](http://localhost:16686/)
Prometheus is at [http://localhost:9090/](http://localhost:9090/)

### Multiple operators

_STATUS: TODO_
```bash
# When starting the whole backend
task backend:start-all OPERATORS=N

# If just starting wavs, not chain
task backend:start-wavs OPERATORS=N
```

Make sure you have that number of submission wallets in your `.env`
