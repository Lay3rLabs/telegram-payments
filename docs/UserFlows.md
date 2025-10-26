# Product Flows

## Register to Receive

1. A person opens telegram and starts a chat with the bot account (this can be passed as a link or a QR code as well as a direct search).
2. The bot prompts the user to enter their wallet address (/receive <address>)
3. They receive a message acknowleging they have a "open account". (From WAVS aggregator)
4. If there are any pending payments (see [Send Payment](#send-payment)), the money will automatically be transferred to their registred address as part of this registration process.

## Register to Send

(This is optional, not needed to just receive)

1. After creating an "open account", the bot will suggest they can fund the account if they wish to make Telegram payments.
2. The person clicks a button to open a mini-app 
3. Prompt to connect wallet
4. Ensure that the wallet address matches the address they registered in the last step, or show a warning to change account.
5. Show the current balance of their account (that they [registered above](#register-to-receive)) and asks how much they want to set as a sending limit
6. Create a transaction to grant the account access to send payments as well as back-link this telegram account from the blockchain address
7. They receive a notification that they now have a funded account (From WAVS aggregator)

## Top Up Account

1. A person with a "funded account" can open up the Mini-App again.
2. It will show them their status (like the /status query), and ask if they want to add more assets to their account.
3. They can select the amount to add and connect their wallet
4. They sign a transaction to grant more balance to the payment rails
5. They receive a notification of their new payment limit (From WAVS aggregator)

## Defund Account

If a person wants to stop using the app and not leave assets exposed to the WAVS operators (eg if Telegram got hacked), they can defund their account.

1. A person with a "funded account" can open up the Mini-App again.
2. It will show them their status (like the /status query), and provide a button to close their account.
3. They connect their wallet and sign a transaction to remove the grant from the payment rails (essentially revoking access to the payment rails)
4. They receive a notification that they now have an "open" account (From WAVS aggregator)

## Status Check

At any time a Telegram user can ping the bot with /status to check their account status. This includes "open" "funded" or "no" account as well as the blockchain address, the balance, and the sending limit.

## Send Payment

This can only be performed by a user with a "funded account"

1. A person opens telegram and starts a chat with the bot account (this can be passed as a link or a QR code as well as a direct search).
2. They user enters /send <receipient_name> <amount> <asset> 
3. WAVS operators pick up the transaction
  a. They checks if the user has a funded account and if the amount is within the sending limit
  b. A quorum of the WAVS operators create a transaction to move the funds to the user's account
4. The payment rails contract efects the transfer
  a. If recipient has an open account, moves funds to the registered blockchain address
  b. If not, recipient is the vault contract address, held to be claimed by the telegram handle when they open an account
5. WAVS aggregator sends a message to the user with a link to the successful transaction, or an error message if it fails

## Payment Notifications

When any payment is sent to a user with an open account, they will receive a message from the bot informing them of the payment.

* If the sender was registered on the payment rails, it will say something like "You just received 35.6 $NTRN from @CryptoCito"
* If the sender was not registered on the payment rails, it will say something like "You just received 35.6 $NTRN from ntrn1d73d0a9bc08a..."

## Technical Notes

The backend is split between a normal Telegram bot (single process), that handles non-secure user interactions quickly, and a secure backend (network of WAVS operators) that handles secure user interactions. 

The Telegram bot:

* Provides basic instructions in the chat, help menu, and links the mini-app URL.
* Handles /status
* Listens for any payment transactions and sends notification messages

WAVS Operators and Aggregator:

* Handles /receive command to register an account as "open"
* Handles /send command to send payments

Mini-App and user wallet talk directly to the blockchain to:

* Register and fund their account
* Defund their account
* Top Up their account