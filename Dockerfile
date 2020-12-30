FROM ubuntu:focal

RUN apt-get update -y

RUN apt-get install -y gcc g++

RUN apt-get install -y software-properties-common && \
    add-apt-repository -y ppa:deadsnakes/ppa && \
    apt-get install -y python3.8 python3-pip

RUN rm -rf /var/lib/apt/lists/*
