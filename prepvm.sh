#!/bin/bash

sudo apt update
sudo apt install squid build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
mkdir dev
pushd dev
git clone https://github.com/brianduff/penguin.git
pushd penguin
cargo build
