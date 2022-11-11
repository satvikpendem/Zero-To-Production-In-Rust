#!/bin/bash
if [[ "$(uname)" == "Linux" ]]; then
    if sudo service postgresql status | rg down; then
        sudo service postgresql restart
    fi
fi

if [[ "$(uname)" == "Darwin" ]]; then
    if brew services info postgresql@15 | rg "Running: false"; then
        brew services restart postgresql@15
    fi
fi
