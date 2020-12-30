docker run --name nb_sandbox ubuntu:focal
docker cp nb_sandbox:/ ./rootfs
docker stop nb_sandbox
docker rm nb_sandbox
