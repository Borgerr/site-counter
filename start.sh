#!/bin/env bash

docker build -t site-counter .
docker run -it --rm --name site-counter site-counter

