# Deploying

See [GettingStarted.md](./GettingStarted.md) for initial setup instructions.

## Tapping the faucet

You probably need to tap the faucet to get some tokens

```bash
task deploy:tap-faucet
```

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

Replace `payments-instantiation.json` with the desired output filename

```bash
task deploy:contract-instantiate-payments FILENAME=payments-instantiation.json CODE_ID={value}
```

This will instantiate the contract and write the address in a JSON file under `builds/deployments/payments-instantiation.json`

## Deploying comopnents

### Uploading

Upload and get an IPFS CID for each component

Replace `operator` and `operator-cid.json` with the contract name and desired output filename

```bash
task deploy:upload-component COMPONENT=operator FILENAME=operator-cid.json
```

This will upload the component and write the cid in a JSON file under `builds/deployments/operator.cid`
