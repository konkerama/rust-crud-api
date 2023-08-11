FROM rust:1.71 as build

# create a new empty shell project
RUN USER=root cargo new --bin rust-crud
WORKDIR /rust-crud

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
# ENV SQLX_OFFLINE true
COPY ./.sqlx ./.sqlx
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/rust_crud*
RUN cargo build --release

# our final base
# https://stackoverflow.com/questions/73037618/glibc-incompatibility-on-debian-docker
FROM debian:bullseye-slim
# RUN apt-get update && apt install -y openssl
# RUN apt install libc-bin=2.29 libc6=2.29

# copy the build artifact from the build stage
COPY --from=build /rust-crud/target/release/rust-crud .

# set the startup command to run your binary
CMD ["./rust-crud"]
