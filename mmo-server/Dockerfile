FROM rustlang/rust:nightly as build

WORKDIR /usr/src/mmo-server
COPY . .

RUN cargo build --release
RUN mkdir -p /build-out
run cp -r target/release /build-out/
RUN ls -al /build-out/

# Ubuntu 18.04
FROM ubuntu@sha256:5f4bdc3467537cbbe563e80db2c3ec95d548a9145d64453b06939c4592d67b6d

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install ca-certificates libssl-dev net-tools iputils-ping nmap && rm -rf /var/lib/apt/lists/*
COPY --from=build /build-out /
EXPOSE 8080/tcp 8080/udp
CMD mmo-server -- --address 0.0.0.0:8080
