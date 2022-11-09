#!/bin/bash
parent_path=$(
    cd "$(dirname "${BASH_SOURCE[0]}")"
    pwd -P
)
cd "$parent_path"

NAME=zero2prod
PORT=8080

./build.sh $1

# # Remove previous containers by getting the container id then stopping it
# CONTAINER_ID=$(docker ps -a -q --filter ancestor=$NAME --format="{{.ID}}")

# # If CONTAINER_ID is set, then stop it, otherwise don't do anything
# if [[ $CONTAINER_ID ]]; then
#     docker stop $CONTAINER_ID
# fi

# Docker compose version of above, stopping containers
# docker compose down --remove-orphans

# Run containers
docker compose up
# docker compose logs -f --no-log-prefix api | bunyan
# docker compose logs -f database
