We have two bots in this system:

1. MAIN bot - this is the bot users will interact with to send and receive payments
2. OPERATOR bots - this is bot each operator creates to join the dispersion channel and listen to messages

This document is only about the MAIN bot setup.

For operator bots, see [OperatorOnboarding.md](OperatorOnboarding.md)

This is a one-time setup, after which users can start using the bot to send and receive payments.

# MAIN Bot Setup

Create a new bot via https://t.me/BotFather

Go through the normal setup, once you have it, make sure you do the following:

`/setcommands` to set the commands for the bot:

```
status - Show registration status of your account
receive - Register your account to receive payments
send - Initiate a transfer to another telegram user
help - Get help and usage information
```

`/newapp` to create a mini-app for this bot

Point to URL you deployed the backend to

# Set webhook

First, set the required environment variables:

```bash
```

Then run:

```bash
task telegram:set-webhook
```

# Create a group for dispersion messages

This is through the Telegram GUI

# Invite the bot to the group

Also through the GUI

# (Optional) Give the bot admin permissions and only allow it to post

Also through the GUI

# Get the group chat ID

Send a `/groupId` message to the group, and the bot will write it back

Set this in your environment:

```bash
SERVER_TELEGRAM_BOT_GROUP_ID="your-group-chat-id"
```
