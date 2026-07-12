FROM rust AS build

# blazing fast and optimized!!!!!!!!!1111!!!
RUN mkdir -p .cargo && printf '\
[profile.release]\n\
lto = true\n\
codegen-units = 1\n\
opt-level = '3'\n\
panic = "abort"\n\
strip = true\n' > .cargo/config.toml

WORKDIR /app
COPY . .
RUN cargo install --root /tmp --path .
RUN strip /tmp/bin/tarpit

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

COPY --from=build --chown=nonroot:nonroot /tmp/bin/tarpit .
USER nonroot
EXPOSE 5000

ENTRYPOINT ["./tarpit"]
