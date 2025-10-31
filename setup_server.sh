#!/bin/bash

# This is meant for setting up machines for serving the code
sudo apt install -y build-essential libssl-dev pkg-config
sudo snap install task --classic
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/Lay3rLabs/telegram-payments.git

# Copy over the env secrets
# scp .env <machine>/telegram-payments

# Restart this and then run one of the following

### Bot

# curl -sSL https://ngrok-agent.s3.amazonaws.com/ngrok.asc \
#   | sudo tee /etc/apt/trusted.gpg.d/ngrok.asc >/dev/null \
#   && echo "deb https://ngrok-agent.s3.amazonaws.com bookworm main" \
#   | sudo tee /etc/apt/sources.list.d/ngrok.list \
#   && sudo apt update \
#   && sudo apt install ngrok
# ngrok config add-authtoken TOP_SECRET

# task backend:start-server
# task backend:ngrok-start

### Aggregator

# task backend:start-wavs-telemetry
# task backend:start-aggregator

### Operator

# task backend:start-wavs-telemetry
# task backend:start-operator-1

