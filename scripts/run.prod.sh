#!/bin/bash
parent_path=$(
    cd "$(dirname "${BASH_SOURCE[0]}")"
    pwd -P
)
cd "$parent_path"

./build.sh release
docker run -p 8080:8080 --rm zero2prod | bunyan
