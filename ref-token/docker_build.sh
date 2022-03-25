#!/bin/bash
set -e
pushd "$(dirname $0)"
HOST_DIR="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

docker build -t contract-builder .

docker run \
     --mount type=bind,source=$HOST_DIR,target=/host \
     --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
     -i -t contract-builder \
     host/build.sh

