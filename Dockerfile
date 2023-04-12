FROM debian:bullseye as builder

ENV RUNTIME_FOLDER=/root/workspace \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=%%RUST-VERSION%%

WORKDIR ${RUNTIME_FOLDER}

COPY . ${RUNTIME_FOLDER}

RUN apt-get update -y && apt-get install pkg-config libssl-dev build-essential cmake curl -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $CARGO_HOME/env && \
    cargo install --path .

# final docker image

FROM debian:bullseye-slim

ENV RUNTIME_FOLDER=/root/workspace

WORKDIR ${RUNTIME_FOLDER}

COPY --from=builder ${RUNTIME_FOLDER}/.env ${RUNTIME_FOLDER}

COPY --from=builder /usr/local/cargo/bin/core-rusty-api /usr/local/bin

RUN apt-get update -y && apt-get install vim ca-certificates -y && apt-get clean

EXPOSE 1342

VOLUME [ "/root/workspace/runtime" ]

ENTRYPOINT [ "/bin/bash", "-l", "-c" ]

CMD ["core-rusty-api"]

