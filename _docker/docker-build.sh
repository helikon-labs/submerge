#!/usr/bin/env bash
set -e
if [[ $1 == "" ]]
    then
    echo "Version parameter does not exist (eg 0.1.5)."
    exit 1
elif [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Building Submerge Docker images v$1."
else
    echo "Invalid version parameter: $1. Enter a valid semver version (eg 0.1.5)."
    exit 1
fi

# cd to script directory
cd "${0%/*}" || exit

# lib
docker build -t helikon/submerge-lib:"$1" --no-cache --build-arg version="$1" -f ./01-base/01-submerge-lib.dockerfile ..
# base
docker build -t helikon/submerge-base:"$1" --no-cache --build-arg version="$1" -f ./01-base/02-submerge-base.dockerfile ..

# crystal postgres
docker build -t helikon/submerge-crystal-postgres:"$1" --no-cache --build-arg version="$1" -f ./02-crystal/01-submerge-crystal-postgres.dockerfile ..
# crystal
docker build -t helikon/submerge-crystal:"$1" --no-cache --build-arg version="$1" -f ./02-crystal/02-submerge-crystal.dockerfile ..