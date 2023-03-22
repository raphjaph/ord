FROM rust:latest as builder
WORKDIR app
COPY . .

RUN rustup component add rustfmt
#Install protoc
RUN apt-get update -y && apt-get install protobuf-compiler -y

RUN cargo build --release 

FROM ubuntu:latest as runtime
WORKDIR app
COPY --from=builder /app/target/release/ord /usr/local/bin

CMD ["/usr/local/bin/ord","api"]
