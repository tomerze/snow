#! /bin/bash

set -e 

DOCKER_BUILDKIT=1 docker build -t alpine-snow -f $1 .
docker create --name alpine-snow alpine-snow

rm -rf /tmp/alpine-snow
mkdir /tmp/alpine-snow

docker export alpine-snow -o /tmp/alpine-snow/fs.tar

tar -xf /tmp/alpine-snow/fs.tar -C /tmp/alpine-snow

rm -rf /tmp/alpine-snow/fs.tar

rm $2
mksquashfs /tmp/alpine-snow $2 -all-root

docker rm alpine-snow
docker rmi alpine-snow
echo "Done!"
