FROM ubuntu:focal

RUN apt-get update -y

RUN apt-get install -y gcc g++

COPY src/runit.s /tmp/runit.s

RUN gcc /tmp/runit.s -o /usr/bin/runit

RUN rm -rf /var/lib/apt/lists/*
