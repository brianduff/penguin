#!/bin/bash -e

DIR=/opt/penguin

CARGO_TARGET_DIR=target/install cargo build --release

set +e
sudo useradd -r penguin
if [ "$?" -eq "9" ]; then
  echo ""
fi
if [ "$?" -ne "0" ]; then
  echo "Failed to add penguin user"
  exit 1
fi
set -e

sudo mkdir -p /opt/penguin/bin
sudo cp target/install/release/penguin /opt/penguin/bin/
sudo mkdir -p /opt/penguin/conf
sudo mkdir -p /opt/penguin/squid_config.d
sudo cp -f penguin.installed.toml /opt/penguin/penguin.toml

sudo chown -R penguin:penguin /opt/penguin

# Install the systemd service for penguin
set +e
sudo systemctl stop penguin-service
set -e
sudo cp penguin.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl start penguin.service

# Install squid configuration for penguin and HUP squid to pick it up
sudo cp penguin.conf /etc/squid/conf.d/
sudo chown penguin:penguin /etc/squid/conf.d/penguin.conf
kill -HUP $(cat /run/squid.pid)

# Add penguin to the proxy group so it can read log files and squid
# config
sudo usermod -a -G proxy penguin

# Update sudoers to allow the penguin user to HUP squid
sudo cp allow-penguin-hup-proxy /etc/sudoers.d/
