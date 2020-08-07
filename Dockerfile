# -----------------
# Cargo Build Stage
# -----------------

FROM rust:latest as cargo-build
WORKDIR /app

RUN cargo install diesel_cli --no-default-features --features "postgres"
RUN curl https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh -o /usr/local/bin/wait-for-it
RUN chmod +x /usr/local/bin/wait-for-it

COPY Cargo.lock .
COPY Cargo.toml .
COPY caching.rs .
RUN mkdir .cargo
RUN sed -i 's#src/main.rs#caching.rs#' Cargo.toml
RUN cargo build --release
RUN cargo vendor > .cargo/config

RUN sed -i 's#caching.rs#src/main.rs#' Cargo.toml
COPY ./src src
RUN cargo build --release
RUN cargo install --path . --verbose

# -----------------
# Final Stage
# -----------------

FROM debian:stable-slim

RUN apt update && apt install -y libpq-dev 

COPY --from=cargo-build /usr/local/cargo/bin/rbot-discord /bin
COPY --from=cargo-build /usr/local/cargo/bin/diesel /bin
COPY --from=cargo-build /usr/local/bin/wait-for-it /bin/wait-for-it

COPY ./migrations migrations

CMD /bin/bash -c "wait-for-it localhost:5432 && diesel migration run && rbot-discord"
