#!/usr/bin/env bash

set -e

echo "*** Start Substrate node template ***"

cd $(dirname ${BASH_SOURCE[0]})/..

mkdir -p .local

docker-compose down --remove-orphans
docker-compose run --rm --service-ports dev $@