#!/usr/bin/env bash
set -eo pipefail

# Create mount directory for service.
mkdir -p $ZK_FILE_PATH

echo "Mounting Cloud Filestore."
mount -o nolock $FILESTORE_IP_ADDRESS:/$FILE_SHARE_NAME ZK_FILE_PATHR
echo "Mounting completed."

/usr/local/bin/proving-server

# Exit immediately when one of the background processes terminate.
wait -n