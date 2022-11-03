#!/bin/bash
parent_path=$(
    cd "$(dirname "${BASH_SOURCE[0]}")"
    pwd -P
)
cd "$parent_path"

export ENVIRONMENT=PRODUCTION
if [ "$1" == "release" ]; then
    cargo build --release --target-dir ../target
    docker build --tag zero2prod ..
else
    cargo build --target-dir ../target
    docker build --tag zero2prod --file ../Dockerfile.dev ..
fi
unset ENVIRONMENT
