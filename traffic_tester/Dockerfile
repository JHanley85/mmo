FROM rustlang/rust:nightly as build

RUN cargo install cargo-build-deps
WORKDIR /usr/src
RUN git clone https://github.com/Gymmasssorla/anevicon.git && cd anevicon && cargo build --release
RUN wget https://github.com/Gymmasssorla/anevicon/releases/download/v6.0.5/anevicon-x86_64-linux > udptest
RUN chmod +x udptest
RUN apt-get update && apt-get install nano
WORKDIR /usr/src
COPY . .

#RUN cargo install cargo-build-deps

RUN cd /usr/src && USER=root cargo new --bin traffic_tester
COPY Cargo.toml ./Cargo.toml
RUN ls -al

WORKDIR /usr/src

#RUN cargo build-deps --release

#COPY . .

#RUN 
CMD tail -f anything