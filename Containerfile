FROM rust AS build

WORKDIR /app
COPY . .
RUN cargo install --root /tmp --path grainpit_webserver/

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

COPY --from=build --chown=nonroot:nonroot /tmp/bin/grainpit_webserver .
USER nonroot
ENV GRAINPIT_ADDR="0.0.0.0:5000"
EXPOSE 5000

ENTRYPOINT ["./grainpit_webserver"]
