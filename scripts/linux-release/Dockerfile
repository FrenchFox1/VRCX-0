# syntax=docker/dockerfile:1
FROM node:24-bookworm-slim AS node
FROM ubuntu:24.04 AS build

ARG WEBKITGTK_VERSION=2.44.0-2
ARG RUST_TOOLCHAIN=stable
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates build-essential clang cmake curl file g++ git libclang-dev \
    libssl-dev libxdo-dev librsvg2-dev patchelf rpm wget \
    libgtk-3-dev libayatana-appindicator3-dev libfontconfig1-dev libfreetype6-dev \
    libwebkit2gtk-4.1-0=${WEBKITGTK_VERSION} libwebkit2gtk-4.1-dev=${WEBKITGTK_VERSION} \
    libjavascriptcoregtk-4.1-0=${WEBKITGTK_VERSION} libjavascriptcoregtk-4.1-dev=${WEBKITGTK_VERSION} \
    gir1.2-javascriptcoregtk-4.1=${WEBKITGTK_VERSION} gir1.2-webkit2-4.1=${WEBKITGTK_VERSION} \
    && rm -rf /var/lib/apt/lists/*
COPY --from=node /usr/local/ /usr/local/
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain "${RUST_TOOLCHAIN}" \
    && rm /tmp/rustup.sh
ENV APPIMAGE_EXTRACT_AND_RUN=1 HUSKY=0
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY . .
RUN node --input-type=module <<'CONFIG'
import fs from 'node:fs';
import { buildReleaseMeta } from './scripts/prepare-release-version.ts';
const { version } = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const meta = buildReleaseMeta(version);
const config = { bundle: { createUpdaterArtifacts: false } };
fs.writeFileSync('/tmp/linux-package.json', JSON.stringify(config));
if (meta.channel === 'beta') {
    config.version = meta.base_version;
    config.bundle.linux = { rpm: { release: `0.beta.${meta.beta_number}` } };
}
fs.writeFileSync('/tmp/linux-rpm.json', JSON.stringify(config));
CONFIG
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/app/target,sharing=locked \
    export TAURI_UPDATER_PUBLIC_KEY="$(node -p 'require("./src-tauri/tauri.conf.json").plugins.updater.pubkey')" \
    && test -n "$TAURI_UPDATER_PUBLIC_KEY" \
    && rm -rf target/release/bundle \
    && VRCX_0_WEBKIT_BUILD_TARGET=safari17 npm run tauri:build -- \
        --config /tmp/linux-package.json --no-sign --bundles appimage \
    && VRCX_0_WEBKIT_BUILD_TARGET=safari18.2 npm run tauri:build -- \
        --config /tmp/linux-package.json --no-sign --bundles deb \
    && VRCX_0_WEBKIT_BUILD_TARGET=safari18.2 npm run tauri:build -- \
        --config /tmp/linux-rpm.json --no-sign --bundles rpm \
    && mkdir -p /artifacts/appimage /artifacts/deb /artifacts/rpm \
    && cp target/release/bundle/appimage/*.AppImage /artifacts/appimage/ \
    && cp target/release/bundle/deb/*.deb /artifacts/deb/ \
    && cp target/release/bundle/rpm/*.rpm /artifacts/rpm/

FROM scratch AS artifacts
COPY --from=build /artifacts/ /
