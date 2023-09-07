FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/placey ./placey

# use a node image for building the site
FROM node:16 as static
WORKDIR /ui
COPY ./src/ui ./src/ui
COPY ./meta ./meta
COPY package.json .
COPY package-lock.json .
COPY tailwind.config.js .
RUN npm i && npm run build:css

# Runtime image
FROM rust:latest
# Run as "app" user
RUN useradd -ms /bin/bash app
USER app
WORKDIR /app
# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/placey /app/placey
COPY --from=static /ui/dist /app/dist
COPY --from=static /ui/meta /app/meta

# Run the app
CMD ./placey