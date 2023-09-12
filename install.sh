#!/bin/bash -e

DIR=/opt/penguin

if [ -d "$DIR" ]; then
  # Take action if $DIR exists. #
  echo "Already exists in $DIR. Will not reinstall"
  exit 0
fi

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

sudo cp penguin.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl start penguin.service
