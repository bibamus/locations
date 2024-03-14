FROM debian:stable-slim
WORKDIR /app

ENV PORT=8080

COPY target/release/backend /app/app

EXPOSE 8080
ENTRYPOINT ["/app/app"]