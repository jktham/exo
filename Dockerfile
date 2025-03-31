FROM rust:1.85.1

COPY ./src/ ./src/
COPY ./.cargo/ ./.cargo/
COPY ./Cargo.toml ./Cargo.toml
COPY ./run-wasm/ ./run-wasm/

RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-server-runner

ENV WASM_SERVER_RUNNER_ADDRESS=0.0.0.0:8000
CMD ["cargo", "run", "--target", "wasm32-unknown-unknown", "--release"]
EXPOSE 8000
