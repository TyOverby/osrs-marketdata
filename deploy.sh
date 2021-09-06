#!/bin/bash
set -euo pipefail

# Kill the server so that we can upload the new package
ssh marketdata@159.65.40.110 'killall "osrs-ge-collect serve"'

# Build the target
cargo build --release

# Upload package
scp target/release/osrs-ge-collect marketdata@159.65.40.110:./

# Restart the server.  The redirection is necessary because otherwise stdout / stderr
# will keep the ssh session open.
ssh marketdata@159.65.40.110 'nohup ./osrs-ge-collect serve 1>/dev/null 2>/dev/null &'