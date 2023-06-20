FROM rust:1.70.0 AS build

COPY ./gurama/ /usr/src/gurama/
RUN cd /usr/src/gurama/ \
    && cargo build --release

FROM debian:bullseye-slim

ENV GURAMA_MODEL_PATH=/usr/src/openbuddy-openllama-7b-v5-q4_0.bin

COPY --from=build /usr/src/gurama/target/release/gurama /usr/src/gurama
COPY ./model/openbuddy-openllama-7b-v5-q4_0.bin /usr/src/openbuddy-openllama-7b-v5-q4_0.bin

ENTRYPOINT [ "/bin/sh", "-c" , "/usr/src/gurama" ]
