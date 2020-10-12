FROM clux/muslrust:nightly-2020-10-11 as build
RUN apt-get update
RUN apt-get -y install clang llvm-dev libclang-dev

RUN mkdir src/
RUN echo "fn main() {println!(\"build failed\")}" > src/main.rs
COPY Cargo.toml Cargo.toml
RUN cargo build -Z unstable-options --out-dir /build
RUN rm -f /build/crypto-timestamp-api; rm -f src/main.rs

COPY diesel.toml diesel.toml
COPY ./src ./src

RUN cargo build -Z unstable-options --out-dir /build --release
RUN ls -l /build
RUN strip /build/crypto-timestamp-api
RUN ls -l /build


FROM scratch
COPY --from=build /build/crypto-timestamp-api /bin/entrypoint
EXPOSE 8080
ENTRYPOINT ["/bin/entrypoint"]


