docker build . -t nb_sandbox
docker run --name nb_sandbox nb_sandbox
rm -rf rootfs
docker cp nb_sandbox:/ ./rootfs
docker stop nb_sandbox
docker rm nb_sandbox

chmod -R 755 rootfs
mknod -m 0666 rootfs/dev/null c 1 3
