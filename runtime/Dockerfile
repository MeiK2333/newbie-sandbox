FROM ubuntu:focal

ENV LANG C.UTF-8

RUN apt-get update -y

# install gcc g++
RUN apt-get install -y gcc g++

COPY runit.s /tmp/runit.s

RUN gcc /tmp/runit.s -o /usr/bin/runit

# install python3.8
RUN apt-get install -y software-properties-common && add-apt-repository -y ppa:deadsnakes/ppa && apt-get install -y python3.8 python3-pip

# install rust
RUN apt-get install -y curl && curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable

# install node
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash - && apt-get install -y nodejs

# node compiler
COPY plugins /plugins
RUN cd /plugins/node && npm install

# node runtime
COPY node /
RUN cd / && npm install

# install typescript
RUN npm install -g ts-node typescript

# install go
RUN add-apt-repository -y ppa:longsleep/golang-backports && apt-get install -y golang-go

# install openjdk
RUN apt-get install -y default-jdk

RUN rm -rf /var/lib/apt/lists/*
