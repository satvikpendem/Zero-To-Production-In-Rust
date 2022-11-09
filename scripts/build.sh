#!/bin/bash
parent_path=$(
    cd "$(dirname "${BASH_SOURCE[0]}")"
    pwd -P
)
cd "$parent_path"

NAME=zero2prod

export ENVIRONMENT=PRODUCTION
if [ "$1" == "release" ]; then
    cargo build --release --target-dir ../target
    docker build --tag $NAME ..
else
    cargo build --target-dir ../target
    docker build --tag $NAME --file ../Dockerfile.dev ..
fi
unset ENVIRONMENT
