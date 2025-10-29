# Deploying

See [GettingStarted.md](./GettingStarted.md) for initial setup instructions.

This document assumes you have already:

1. Started the backend

```bash
task backend:start-all
```

2. Funded the wallets

```bash
task deploy:tap-faucet-all
```

3. Built everything:

```bash
task contracts:build-all && task components:build-all
```

Then it's as simple as this to deploy everything

```bash
task deploy:all
```
Alternatively, skip uploading contracts and/or components
this assumes they were already uploaded before and the output files exist

```bash
task deploy:all SKIP_UPLOAD_CONTRACTS=true SKIP_UPLOAD_COMPONENTS=true
```

What's this all doing?

Ultimately, deploying consists of the following steps:

1. Upload middleware service manager contract to code id
2. Instantiate middleware service manager contract to address
3. Upload our contracts to get code IDs
4. Instantiate our contracts to get addresses
5. Upload components to get IPFS CIDs
6. Upload services to get IPFS CIDs
7. Set the service URI on the service manager contract
8. Add the service manager address to aggregator node
9. Add the service manager address to wavs nodes


Each of these steps with independent commands is described below.

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

## Set the service URI on the service manager contract

The service manager address is obtained from either the middleware instantiation step or by looking at the service.json itself

```bash
task deploy:middleware-set-service-uri ADDR=<service-manager-address> URI=<service-uri>
```

## Register the service on the aggregator

The service manager address is obtained from either the middleware instantiation step or by looking at the service.json itself

```bash
task deploy:aggregator-register-service ADDR=<service-manager-address>
```

## Add the service to the operator

The service manager address is obtained from either the middleware instantiation step or by looking at the service.json itself

```bash
# alternatively, if you have several operators
# task deploy:operator-add-service ADDR=<service-manager-address> OPERATORS=3
task deploy:operator-add-service ADDR=<service-manager-address>
```
