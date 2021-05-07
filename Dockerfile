FROM rustlang/rust:nightly as builder

WORKDIR /
RUN cargo new --bin bnuuy

WORKDIR /bnuuy
COPY ./Cargo.toml ./Cargo.toml
# TODO optionally get rid of release
RUN cargo build --release 
RUN rm -rf src/

ADD . ./

RUN cargo build --release


FROM python:3.9

WORKDIR /bnuuy
RUN pip install instagram-scraper==1.9.1
COPY --from=builder /bnuuy/target/release/bnuuy /bnuuy/bnuuy
COPY ./Rocket.toml ./Rocket.toml

VOLUME /db
ENV DATABASE_URL=/db/dogs.sqlite
CMD ["/bnuuy/bnuuy"]
