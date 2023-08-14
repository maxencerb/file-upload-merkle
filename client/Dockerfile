FROM rust:1.67

ENV SERVER=http://localhost:3000

WORKDIR /usr/src/fum-client
COPY . .

RUN cargo install --path .

CMD [ "fum-client" ]