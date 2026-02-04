#!/bin/bash

set -euxo pipefail

if ! docker info > /dev/null 2>&1; then
  echo "🐳 This script uses Docker, and it isn't running - please start Docker and try again!"
  exit 1
fi

docker compose -p submerge -f submerge.yml down