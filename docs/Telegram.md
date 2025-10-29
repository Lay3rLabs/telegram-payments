# Telegram Integration

We want the following:

1. A user can send messages which are received BOTH by our backend server (bot) as well as all operators
2. THe backend server (bot) uses webhooks for quick responses
3. The backend server (bot) can send messages to users
4. The aggregator can send messages to users

For this prototypes, we can share API keys among multiple actors. Later, ideally more fine grained.

## Problems

### WebHooks not working

First approach was to share the same Telegram API key with all machines. It hit a few issues:

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
That said, it does seem that many different machines can poll with the same API key, they just need to manage offsets well

### Relay in a Group

We thought to make multiple telegram bots (one for our backend server and one for the operators), and have them relay messages to each other.
However, this is also not possible. (As well as being a potentail security issue)

https://core.telegram.org/bots/faq#why-doesn-39t-my-bot-see-messages-from-other-bots

```
Why doesn't my bot see messages from other bots?
Bots talking to each other could potentially get stuck in unwelcome loops. To avoid this, we decided that bots will not be able to see messages from other bots regardless of mode.
```

## Proposed Solutions

### Long Polling

If we have just one Telegram Bot account and one API key shared with all, we can use long polling to receive updates.
We just remove requirement 2, and have the backend server also use long polling for this.
And we stay close to the original design

### Payment Group

In this case, we have one API key for the backend server (read-write) and aggregator (write-only). We have a second TG bot and API key shared among all operators for reading. THis bot should be mute, or at least have a funny name, so no one wants to send messages from it. Let's call these "Payment Bot" and "Operator Bot"

A user of the service, will interact with a group containing them and both bots. The "Payment Bot" writing to them, and the "Operator Bot" allowing all operators to listen in.

How do we create this easily? Plan:

1. User starts a chat with the Payment Bot (/start)
2. Payment Bot creates a group and invites the Operator Bot
3. Payment Bot sends a link to the user to join the group
4. User sends commands to this group.

Questions:

1. Can a bot create a group and add people?
2. Can we embed mini-apps to a group? Or only in the direct chat?

## Plan

I think the "Payment Group" is the most secure yet practical solution for scaling out later. However, it is not the most user-friendly, nor is it clear if it will work or hit another Telegram API issue.

For the Hackathon, let's just use Long Polling everywhere and see if that works.
