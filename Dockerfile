FROM rust:latest

RUN apt-get update && apt-get install -y tini && rm -rf /var/lib/apt/lists/*

EXPOSE 80

WORKDIR /app
ENTRYPOINT ["/usr/bin/tini", "--"]