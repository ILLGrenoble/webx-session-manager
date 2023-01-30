FROM rustlang/rust:nightly-slim
WORKDIR /app

RUN apt update
RUN apt install -y libzmq3-dev pkg-config dpkg-dev libclang-dev libpam-dev clang

COPY . .

RUN cargo install cargo-deb
RUN cargo deb

# Save the version to a file
RUN awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' Cargo.toml > VERSION

# to obtain built deb package:
# docker build -t webx-sesman-builder .
# docker create -ti --name webx-sesman-builder webx-sesman-builder bash
# docker cp webx-sesman-builder:/app/target/debian/. .
# docker rm -f webx-sesman-builder

