#!/bin/bash
parent_path=$(
    cd "$(dirname "${BASH_SOURCE[0]}")"
    pwd -P
)
cd "$parent_path"

./build.sh $1
docker run -p 8080:8080 --rm zero2prod | bunyan
