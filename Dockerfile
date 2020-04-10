   FROM rust:buster AS builder
WORKDIR /app
    RUN apt-get update && apt-get install -y llvm-dev libclang-dev clang
    RUN rustup install nightly && rustup override set nightly
    RUN git clone https://github.com/nothings/stb /usr/local/include/stb

   COPY . .
    RUN cargo install --root /usr/local/cargo --path .

   FROM debian:buster-slim
    RUN apt-get update && apt-get -y upgrade
   COPY --from=builder /usr/local/cargo/bin/simple-uploader /usr/local/bin/simple-uploader
   COPY testpage /testpage
 EXPOSE 3000
    CMD /usr/local/bin/simple-uploader
