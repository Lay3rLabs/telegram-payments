# Telegram Integration

We want the following:

1. A user can send messages which are received BOTH by our backend server (bot) as well as all operators
2. The backend server (bot) uses webhooks for quick responses
3. The backend server (bot) can send messages to users
4. Operators read messages from users by polling.

## Problems

### API Keys are always admin

Telegram does not allow granular permissions for bots. Each bot has one API key, and that key is always admin.

Sharing one API key among multiple machines allows them all to do anything such as delete messages, ban users, change bot settings, etc.

We accept this risk for the purpose of a hackathon prototype, but the design should allow for more granular permissions in the future.

### WebHooks not working

Even if we share the same Telegram API key with all machines, we have a few issues:

https://core.telegram.org/bots/api#getupdates

```
getUpdates
Use this method to receive incoming updates using long polling
...
Notes
1. This method will not work if an outgoing webhook is set up.
2. In order to avoid getting duplicate updates, recalculate offset after each server response.
```

So, if we want to use the webhook, we cannot use the same API key to poll.

### Long Polling not working

Even if we remove the webhook from the backend server and use long-polling everywhere, we have another issue:

https://core.telegram.org/bots/api#getupdates

```
An update is considered confirmed as soon as getUpdates is called with an offset higher than its update_id.
[...]
All previous updates will be forgotten.
```

So if we have multiple machines using the same API key to poll, they will step on each other's toes, and miss messages.


### Relay in a Group

We thought to make multiple telegram bots (one for our backend server and one for the operators), and have them relay messages to each other.

However, this is also not possible. (As well as being a major security issue since the server can manipulate messages)

https://core.telegram.org/bots/faq#why-doesn-39t-my-bot-see-messages-from-other-bots

```
Why doesn't my bot see messages from other bots?
Bots talking to each other could potentially get stuck in unwelcome loops. To avoid this, we decided that bots will not be able to see messages from other bots regardless of mode.
```

## Proposed Solution

### Payment Group

In this case, we have different kinds of bots:

1. "Messenger Bot": This is a single bot the powered by webhooks, deals with cosmetic messaging only. It requires specialized software and runs on a traditional server.
2. "Operator Bots": These aren't really bots, rather they are keys operators use to read messages only. Each operator has their own bot key.

A user of the service will interact with a group containing all of these bots. Users send messages to the group and the bots react accordingly:

1. "Messenger Bot": handles cosmetic messaging such as welcome messages, giving instructions, private messaging, etc.
2. "Operator Bots": read messages from users and process them (e.g. approve payments).

#### The flow is as follows:

Receive Flow:

1. A user messages the "Messenger Bot" in private chat (link to start can be provided via QR code)
2. The "Messenger Bot" creates an invite link to a group containing itself and all the operator bots
3. The user joins the group via the invite link
4. The "Messenger Bot" greets the user in the group, asking them to type `/receive <addr>`
5. User onboards with `/receive` which is processed by the "Operator Bots" (See payment flow step 2)
6. The operator bots also transfer any unclaimed funds to users when they enter receive (happens automatically in the CW contract)

Funding Flow:

1. "Messenger Bot" prompts the user with a link for the mini-app funding registration. Something like `https://t.me/botusername?startapp` from [Mini App Docs](https://core.telegram.org/bots/webapps#launching-the-main-mini-app)
2. The "Messenger Bot" onboards the user via the mini-app, where they connect their wallet and send a dual-message transaction:
  a. Authz grant for payments
  b. Registration message to link their Telegram handle to their blockchain address

Payment Flow:

1. The user sends payment commands in the group (e.g. /send @recipient 10 NTRN)
2. All operators read the messages in the group and process them accordingly.
  a. This is via WAVS components and their individual bot keys
  b. Ultimately signatures are aggregated and sent to the service handler contract
  c. If a user is registered (i.e. in the group), payment is transferred, otherwise it's set aside for claim

**Questions**: Can we use command menus like /help in the group chat? Do you need to mention the bot? Does the bot read everything? If I start typing `/s` do I get inline help like in the DM chat?

However, there is one major problem:

The Bot API disallows creating groups on-the-fly :(

For the purpose of the hackathon, this means all users are in one group together, which is not the ideal experience. It also means that operator bots must be added manually by a human admin.

Humans can join the group via an automatically generated invite link from the Messenger Bot, however

### Scaling for production

In production, the `MTProto`/`grammers` APIs can be used to create groups and add people programmatically.

This has several advantages:

1. Each user can have their own private group with the bots, improving privacy and security.
2. Bots can be added programmatically, improving usability.
3. Groups can be created and deleted automatically

However, this requires setting up a real user account tied to a personal phone number and is out of scope for the hackathon.

At scale, due to the restrictions Telegram places on this approach, it may be necessary to have multiple user accounts (and phone numbers) to create groups for new users due to rate restrictions and/or coordinate with Telegram business offerings to ensure smooth operation.
