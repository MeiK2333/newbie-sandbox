FROM ubuntu:focal

ENV LANG C.UTF-8

RUN apt-get update -y

# install gcc g++
RUN apt-get install -y gcc g++

COPY runit.s /tmp/runit.s

RUN gcc /tmp/runit.s -o /usr/bin/runit

# install python3.8
RUN apt-get install -y software-properties-common && \
    add-apt-repository -y ppa:deadsnakes/ppa && \
    apt-get install -y python3.8 python3-pip

# install rust
RUN apt-get install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable

# install node
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash - && \
    apt-get install -y nodejs

COPY plugins /plugins
# node compiler
RUN cd /plugins/node && npm install

# node runtime
COPY node /
RUN cd / && npm install

# install typescript
RUN npm install -g ts-node typescript

# install go
RUN add-apt-repository -y ppa:longsleep/golang-backports && \
    apt-get install -y golang-go

# install openjdk
RUN apt-get install -y default-jdk

# install c#(mono)
RUN apt-get install -y gnupg ca-certificates && \
    apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF && \
    echo "deb https://download.mono-project.com/repo/ubuntu stable-focal main" | tee /etc/apt/sources.list.d/mono-official-stable.list && \
    apt update && \
    apt-get install -y mono-devel

# install php
RUN add-apt-repository -y ppa:ondrej/php && apt-get install -y php8.0-fpm

# install ruby
RUN apt-get install -y ruby-full

# install perl
RUN apt-get install -y perl

# install haskell
RUN add-apt-repository -y ppa:hvr/ghc && apt-get install -y cabal-install ghc

# install pascal
RUN apt-get install -y fpc

RUN rm -rf /var/lib/apt/lists/*
