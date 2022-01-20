FROM rust:1.57-slim

WORKDIR /app

RUN apt update
RUN apt install -y libzmq3-dev pkg-config dpkg-dev libclang-dev libpam-dev

COPY . .

RUN cargo install cargo-deb
RUN cargo deb

# to obtain built deb package:
# docker build -t webx-sesman-builder .
# docker create -ti --name webx-sesman-builder webx-sesman-builder bash
# docker cp webx-sesman-builder:/app/target/debian/. .
# docker rm -f webx-sesman-builder
