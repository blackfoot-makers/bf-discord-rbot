FROM rust:latest
RUN curl https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh -o /usr/local/bin/wait-for-it
RUN chmod +x /usr/local/bin/wait-for-it

WORKDIR /discord-rbot
COPY . .
RUN cargo install diesel_cli --no-default-features --features "postgres"
RUN cargo build --release

CMD /bin/bash -c "wait-for-it localhost:5432 && diesel migration run && cargo run --release"