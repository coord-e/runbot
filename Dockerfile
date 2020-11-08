FROM rust:slim-buster

WORKDIR /build

RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl

RUN apt-get update -y
RUN apt-get install -y musl-tools

COPY . /build/
RUN cargo build --release --target=x86_64-unknown-linux-musl


FROM alpine

COPY --from=0 /build/target/x86_64-unknown-linux-musl/release/runbot-discord /usr/bin/runbot-discord
COPY --from=0 /build/table.toml /etc/runbot/table.toml

ENV RUNBOT_TABLE_FILE_PATH=/etc/runbot/table.toml

CMD ["/usr/bin/runbot-discord"]
