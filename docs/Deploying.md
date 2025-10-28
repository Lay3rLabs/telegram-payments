# Deploying

See [GettingStarted.md](./GettingStarted.md) for initial setup instructions.

This document assumes you have already:

1. Built everything with `task contracts:build-all` and `task components:build-all`
2. Started the backend with `task backend:start-all`
3. Funded the wallet (you can do that with `task deploy:tap-faucet`)


Ultimately, deploying consists of the following steps:

1. Upload middleware service manager contract to code id
2. Instantiate middleware service manager contract to address
3. Upload our contracts to get code IDs
4. Instantiate our contracts to get addresses
5. Upload components to get IPFS CIDs
6. Upload services to get IPFS CIDs
7. Add the service manager address to aggregator node
8. Add the service manager address to wavs nodes

Let's go!

## Deploying middleware service manager

### Uploading

Upload and get a code ID

```bash
task deploy:middleware-service-manager-upload FILENAME=service-manager-code-id.json
```

This will upload the contract and write the code-id in a JSON file under `builds/deployments/service-manager-code-id.json`

Stake registry is similar:

```bash
task deploy:middleware-stake-registry-upload FILENAME=stake-registry-code-id.json
```

This will upload the contract and write the code-id in a JSON file under `builds/deployments/stake-registry-id.json`

### Instantiating

Instantiate using the code IDs from the previous steps

```bash
task deploy:middleware-instantiate MANAGER_CODE_ID={value} REGISTRY_CODE_ID={value} FILENAME=middleware.json
```

This will instantiate the stake registry and manager, and write the addresses in a JSON file under `builds/deployments/middleware.json`

You can override the default threshhold weight and strategy parameters by setting `THRESHOLD` and `STRATEGIES` environment variables.

## Deploying contracts

### Uploading

Upload and get a code ID for each contract

Replace `payments` and `payments-code-id.json` with the contract name and desired output filename

```bash
task deploy:contract-upload CONTRACT=payments FILENAME=payments-code-id.json
```

This will upload the contract and write the code-id in a JSON file under `builds/deployments/payments-code-id.json`

### Instantiating

Instantiate using the code ID from the previous step

Contract instantiations use specific commands per contract.

```bash
task deploy:contract-instantiate-payments CODE_ID={value} FILENAME=payments.json
```

This will instantiate the contract and write the address in a JSON file under `builds/deployments/payments.json`

## Uploading components

Upload and get an IPFS CID for each component

```bash
task deploy:component-upload COMPONENT=operator FILENAME=operator-cid.json
task deploy:component-upload COMPONENT=aggregator FILENAME=aggregator-cid.json
```

This will upload the component and write the cid in a JSON file under `builds/deployments/operator-cid.json` and `builds/deployments/aggregator-cid.json` respectively.

## Uploading services

Upload and get an IPFS CID for the service

Each of the variables are the filenames set in the previous steps, except for `FILENAME` which is the desired output filename for the service CID

```bash
task deploy:service-upload \
    CONTRACT_PAYMENTS=payments.json \
    MIDDLEWARE=middleware.json \
    COMPONENT_OPERATOR=operator-cid.json \
    COMPONENT_AGGREGATOR=aggregator-cid.json \
    FILENAME=service-cid.json
```

This will upload the service and write the cid in a JSON file under `builds/deployments/service-cid.json`

The full service JSON is also written to that file under the `service` key
