# Telegram Payments - Frontend

A Telegram Mini App for managing Cosmos accounts and making payments via Telegram.

## Tech Stack

- React 18 with TypeScript
- Vite for build tooling
- Telegram Web App SDK (@twa-dev/sdk)
- Mock crypto services (ready for Cosmos integration)
- Make sure to run node 22.12+ for vite

## Development

### Prerequisites

- Node.js 18+ (Note: Some dependencies show engine warnings with older versions, but should still work)
- npm or yarn

### Installation

```bash
cd frontend
npm install
```

### Running Locally

```bash
npm run dev
```

The app will be available at `http://localhost:5173`

### Testing in Telegram

**Quick Start:**

1. Start the dev server: `npm run dev`
2. Expose it with [ngrok](https://ngrok.com/): `ngrok http 5173`
3. Create/configure a bot with [@BotFather](https://t.me/botfather)
4. Set your ngrok HTTPS URL as the Web App URL
5. Test the app in Telegram

**Three Ways to Launch Your Mini App:**

1. **Menu Button** (Recommended) - Always visible button next to message input
2. **Direct Link** - Shareable URL like `t.me/your_bot/appname`
3. **Inline Buttons** - Buttons in bot messages (requires bot backend)

📖 **See [BOT_SETUP.md](BOT_SETUP.md) for detailed setup instructions with screenshots and step-by-step guides for each method.**

### Building for Production

```bash
npm run build
```

The built files will be in the `dist` directory.

### Storage Service (`src/services/storage.ts`)

Manages account persistence in localStorage:

- Saves public key and address (NOT private key)
- Private key shown only once during creation
- User responsible for backing up private key

## Environment Variables

You can create a `.env` file for configuration:

```env
VITE_CHAIN_ID=cosmoshub-4
VITE_RPC_ENDPOINT=https://rpc.cosmos.network
VITE_API_ENDPOINT=https://api.cosmos.network
```
