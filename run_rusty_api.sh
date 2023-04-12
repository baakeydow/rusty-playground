#!/usr/bin/env bash

ROOT_DIR="$( cd -P "$( dirname "$SOURCE" )" >/dev/null 2>&1 && pwd )"

# delete old docker image if any
docker rmi -f rusty_core:latest &>/dev/null || true

# stop old container if any
docker stop running_rusty_api &>/dev/null || true

# build fresh image and run new container in detached mode
docker build --no-cache --pull -t rusty_core:latest . && \
  docker run --rm --name running_rusty_api -d \
  -p 1342:1342 \
  -v "$ROOT_DIR/runtime:/root/workspace/runtime" rusty_core

# display logs
docker logs -f running_rusty_api
