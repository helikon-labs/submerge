#!/usr/bin/env bash
set -e
if [[ $1 == "" ]]
    then
    echo "Version parameter does not exist (eg 0.1.5)."
    exit 1
elif [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Building and publishing SubVT Docker images v$1."
else
    echo "Invalid version parameter: $1. Enter a valid semver version (eg 0.1.5)."
    exit 1
fi

# cd to script directory
cd "${0%/*}" || exit

# backend base
docker build -t helikon/submerge-base:"$1" -t helikon/submerge-base:latest --no-cache --build-arg version="$1" -f ./base/02-submerge-base.dockerfile ..
# backend lib
docker build -t helikon/submerge-lib:"$1" -t helikon/submerge-lib:latest --no-cache --build-arg version="$1" -f ./base/01-submerge-lib.dockerfile ..