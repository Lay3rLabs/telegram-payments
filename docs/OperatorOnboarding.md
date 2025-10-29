We have two bots in this system:

# Bot Setup

Create a new bot via https://t.me/BotFather

NOTE: You must turn group privacy off for the bot via BotFather:

```
/mybots -> choose your bot -> Bot Settings -> Group Privacy -> Turn off
```


Go through the normal setup and make sure you set the received API key in your environment as:

```bash
OPERATOR_TELEGRAM_BOT_API_KEY="your-telegram-bot-api-key"
```

# Join Dispersion Channel

Each operator bot needs to join the dispersion channel to listen to messages.

As of right now, this is a manual process, ask the the owner of the MAIN bot group to add your bot
