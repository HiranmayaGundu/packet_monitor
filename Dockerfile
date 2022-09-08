FROM ubuntu:16.04

RUN apt-get update && apt-get install -y build-essential ca-certificates curl
RUN curl --proto "=https" --tlsv1.2 --retry 3 -sSfL https://sh.rustup.rs | sh -s -- -y