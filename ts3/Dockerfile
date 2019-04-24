# Base system is Ubuntu 16.04
FROM ubuntu:16.04 as build

# Set the Teamspeak version to download
ENV TSV=3.2.0

# Download and install everything from the repos.
RUN    DEBIAN_FRONTEND=noninteractive \
        apt-get -y update && \
        apt-get -y install bzip2 ca-certificates build-essential && \
        rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
        apt-get autoremove -y && \
        apt-get clean

# make build-base libc6-compat gcc linux-headers musl-dev
# RUN set -eux; \
#  addgroup -g 9987 ts3server; \
#  adduser -u 9987 -Hh /var/ts3server -G ts3server -s /sbin/nologin -D ts3server; \
#  mkdir -p /var/ts3server /var/run/ts3server; \
#  chown ts3server:ts3server /var/ts3server /var/run/ts3server; \
#  chmod 777 /var/ts3server /var/run/ts3server

ENV PATH "${PATH}:/opt/ts3server"

#ARG TEAMSPEAK_CHECKSUM=c82eebbe5dca9f33c8e52b0526fb92613f975c73463687e1561eefb79c0e5a69
#ARG TEAMSPEAK_URL=https://files.teamspeak-services.com/releases/server/3.7.1/teamspeak3-server_linux_alpine-3.7.1.tar.bz2
WORKDIR /opt/
COPY . /opt/ts3_server
WORKDIR /opt/ts3_server/Examples/server
RUN  make -f Makefile.linux_amd64 
# RUN set -eux; \
#  apk add --no-cache --virtual .fetch-deps tar; \
#  wget "${TEAMSPEAK_URL}" -O server.tar.bz2; \
#  echo "${TEAMSPEAK_CHECKSUM} *server.tar.bz2" | sha256sum -c -; \
#  mkdir -p /opt/ts3server; \
#  tar -xf server.tar.bz2 --strip-components=1 -C /opt/ts3server; \
#  rm server.tar.bz2; \
#  apk del .fetch-deps; \
#  mv /opt/ts3server/*.so /opt/ts3server/redist/* /usr/local/lib; \
#  ldconfig /usr/local/lib; \
#  chown -R ts3server:ts3server /opt/ts3server



FROM centos:latest
COPY --from=build /opt/ts3_server /opt/ts3server
#RUN apt-get update -y && apt-get install -y nano bash
# setup directory where user data is stored
VOLUME /var/ts3server/
WORKDIR /var/ts3server/
COPY ./log.sh /opt/log.sh
RUN chmod 777 /opt/ts3server/Examples/ -R
#  9987 default voice
# 10011 server query
# 30033 file transport
EXPOSE 9987/udp 10011 30033 
COPY bootstrap.sh /opt/ts3server/bootstrap.sh
# ENTRYPOINT [ "/bin/bash" ]
# CMD ["/bin/bash ./log.sh"]
#CMD ["sh /opt/ts3server/Examples/server/ts3_server_sample"]
