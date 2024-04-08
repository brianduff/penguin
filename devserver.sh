#!/bin/bash

echo "Starting dev server"

# Source and destination directories
SOURCE_DIR="/mnt/penguindev"
DESTINATION_DIR="/home/ubuntu/dev/penguin_dev"

# How often to check for changes (in seconds)
SLEEP_DURATION=1

# Initialize previous checksum
PREV_CHECKSUM=""

while true; do
  # Generate checksum of all files in directory
  CURRENT_CHECKSUM=$(find "$SOURCE_DIR" -type d \( -name node_modules -o -name .jj -o -name .git -o -name target -o -name dist \) -prune -o -type f -exec sha1sum {} + | sort | sha1sum)

  # Compare the previous and current checksum
  if [ "$PREV_CHECKSUM" != "$CURRENT_CHECKSUM" ]; then
    echo "Detected changes - updating server"
    # Sync the directories
    rsync -av --exclude='**/.jj/**' --exclude='**/.git/**' --exclude='**/target/**' --exclude='**/dist/**' --exclude='**/node_modules/**' --delete "$SOURCE_DIR" "$DESTINATION_DIR"
    # Update previous checksum
    PREV_CHECKSUM=$CURRENT_CHECKSUM

    pushd "$DESTINATION_DIR/penguindev/server"
    cargo build --release &&
        sudo service penguin stop &&
        sudo cp target/release/penguin /opt/penguin/bin/ &&
        sudo chown -R penguin:penguin /opt/penguin &&
        sudo service penguin start &&
        pushd ../client &&
        npm install &&
        npm run build &&
        sudo cp -r dist/* /opt/penguin/web
    echo "Listening for updates..."
  fi

  # Wait for the specified duration
  sleep $SLEEP_DURATION
done