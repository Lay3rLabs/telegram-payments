# Telegram Payments

This is an entry for [Hackmos: Cosmoverse 2025](https://dorahacks.io/hackathon/hackmos-2025/detail)

It shows a [WAVS Service](https://www.wavs.xyz) that uses a decentralized operator set to bridge Telegram and Cosmsos.
These operators manage a Telegram bot and trigger on-chain payments when a user requests this from the bot.
They also monitor the users account and can send a Telegram message anytime the user receives a payment.

## Problem Space

The usability of crypto apps for the general public has long been a problem, since people started talking about mass adoption of web3.
One stated goal is to make crypto as usable as credit cards or banking apps. For this project we will create an app like Venmo / Bizzum
that allows anyone to make a bank transfer to anyone else only knowing their telephone number. Under the hood, it runs through the banks.

Telegram is a popular app for crypto holders and recently added a payments interface that will let you send TON or "fiat" on the TON network.
It is surprisingly easy to use and we wish to make something similar but for Cosmos assets. There should be a quick initial setup and then the
ability to quickly and easily transfer many different Cosmos assets to anyone else with a Telegram account.

Critically, we want to make this decentralized and secure, avoiding one central server managing all the wallets and payments, but rather a multi-party
verification required to move the funds on the blockchain.

## Design

* **Ease of Use**: Anyone can register their account with a minimal gas fee and are able to send payments to anyone else with a Telegram account
* **Secure**: A decentralized operator set based on slashable restaked assets verifies the Telegram messages and must reach consensus to move funds 
* **Liquid**: Rather than move funds into some vault to allow the operator network to send them, or signing a transaction for each transfer, the crypto holder must simply sign one grant message to enable the operator network to transfer funds from their account when a proper Telegram message is received.
* **Notifications**: The operators monitor the users account and can send a Telegram message anytime the user receives a payment.

## Implementation

There are three main components in the system:

* **Telegram Bot**: We register a Telegram bot account and generate an API key for it. Query operations will contact a simple backend process running as a bot. The WAVS operators will also have this API key respond to messages for the secure operations.
* **WAVS Operator Set**: A secure, decentralized operator set based on slashable restaked assets verifies the Telegram messages and must reach consensus to move funds. 
* **Payment Rails**: A smart contract is deployed on each supported chain. Anyone wishing to send an asset must make a grant message to the Fund Manager, allowing it to later transfer funds from their account. This handles the actual transfer. It maintains the Telegram account(s) that can send payments for each blockchain address. It also maintains a reverse mapping from blockchain address to Telegram account.

Q: Why do we have two registrations?

A: If providing funds access also provides the reverse registration, there is a clear attack vector. I could register my address and provide say 0.1 $ATOM of funds, claiming CryptoCito's telegram handle. Then anyone trying to send funds to him would end up sending them to me. There are two ways of proving users, either we need a bidirectional proof before any registration, or we need two different registration steps - one to send and one to receive. Signing a transaction (grant message) will say "this address grants access to a telegram handle to move these funds". Sending a special message from telegram to the bot says "this telegram handle wishes to receive any funds at this private key". We could combine them in one flow, but this is not always possible.

For example, if I don't have any $ATOM, and wish to receive donations, I can easily make the claim on Telegram, but I cannot sign an on-chain transaction yet. There should be an easy way to set up to receive, and then a second step to grant access to send. 
