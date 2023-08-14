FROM rust:1.67

ENV SERVER_PORT=3000

WORKDIR /usr/src/fum-server
COPY . .

RUN cargo install --path .

CMD ["fum-server"]