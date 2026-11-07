#!/bin/sh
set -a
source .env.prod
set +a


export $(grep -v '^#' .env.prod | xargs) && \
for var in $(grep -v '^#' .env.prod | cut -d= -f1); do
  gh secret delete "$var"
done
