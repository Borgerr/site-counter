#!/bin/env bash

START_URL="https://wikipedia.com"

TEMP=`getopt --long -o "e:" "$@"`
eval set -- "$TEMP"
while true ; do
    case "$1" in
        -u )
            START_URL=$2
            shift 2
        ;;
        *)
            break
        ;;
    esac
done;

echo "START_URL = $START_URL"

docker build -t site-counter .
docker run -it --rm --name site-counter -e START_URL="$START_URL" site-counter

