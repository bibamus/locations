FROM debian:stable-slim

RUN apt-get update &&	apt-get install -y --no-install-recommends ca-certificates

WORKDIR /app

ENV PORT=8080

COPY target/release/backend /app/app

EXPOSE 8080
ENTRYPOINT ["/app/app"]

