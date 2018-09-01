FROM rustlang/rust:nightly as build

WORKDIR /usr/src/mmo-mumble
COPY . .


RUN cargo build --release
RUN mkdir -p /build-out
RUN mkdir -p /src
run cp -r target/release /build-out/
RUN cp -r /usr/src/mmo-mumble /src
RUN ls -al /build-out/release/


# Ubuntu 18.04
FROM ubuntu:14.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install ca-certificates libssl-dev nano iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=build /build-out /
COPY --from=build /usr/src/mmo-mumble/Config.toml /release/Config.toml
# Setup OpenSSL
RUN apt-get update
RUN apt-get install -y wget
RUN apt-get install -y build-essential
RUN apt-get install -y zlib1g-dev
ARG OPENSSL_VERSION=1.1.0g
RUN wget https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz
RUN tar xvfz openssl-${OPENSSL_VERSION}.tar.gz
RUN cd openssl-${OPENSSL_VERSION} && ./config && make && make install
RUN echo '/usr/local/lib' >> /etc/ld.so.conf
RUN cat /etc/ld.so.conf
RUN ldconfig
RUN echo 'export LD_LIBRARY_PATH=/usr/local/lib' >> ~/.bash_profile && . ~/.bash_profile
RUN openssl version
#CMD mmo-server -- --address 0.0.0.0:8080