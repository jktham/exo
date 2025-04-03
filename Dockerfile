FROM rust:1.85.1

COPY ./deploy/ ./deploy/

RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-server-runner

ENV WASM_SERVER_RUNNER_ADDRESS=0.0.0.0:8000
ENV WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML=./deploy/index.html
CMD ["wasm-server-runner", "./deploy/exo.wasm"]
EXPOSE 8000
