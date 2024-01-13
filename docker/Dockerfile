FROM rust:1.74.0 AS builder

RUN rustup target add wasm32-unknown-unknown

RUN apt-get update && apt-get install -y --no-install-recommends \
        clang \
        libclang-dev \
        protobuf-compiler \
        cmake \
        zlib1g-dev \
        openssl \
        libssl-dev \
        libstdc++6

WORKDIR /sugarfunge

COPY . /sugarfunge

RUN cargo build --locked --release

# Find and copy libssl.so.3 and libcrypto.so.3 libraries
RUN ldd /sugarfunge/target/release/sugarfunge-node | grep libgcc_s.so.1 | awk '{print $3}' | xargs -I {} cp {} /libgcc_s.so.1

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /lib/x86_64-linux-gnu/libz.so.1 /lib/x86_64-linux-gnu/libz.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libssl.so.3 /lib/x86_64-linux-gnu/libssl.so.3
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=builder /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib/x86_64-linux-gnu/
COPY --from=builder /libgcc_s.so.1 /lib/

COPY --from=builder /sugarfunge/target/release/sugarfunge-node /

COPY ./customSpecRaw.json /customSpecRaw.json

ENTRYPOINT ["/sugarfunge-node"]

CMD ["--dev", "--pruning", "archive", "--tmp"]
