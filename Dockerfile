# ---------------------------------------------------
# 1 - Build Stage
#
# Use official rust image to for application build
# ---------------------------------------------------
FROM rust:latest as chef

RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get install -y \
    libpq5

WORKDIR /server

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /server/recipe.json recipe.json
RUN --mount=type=ssh cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin server

# ---------------------------------------------------
# 2 - Deploy Stage
#
# Use a distroless image for minimal container size
# - Copy `libpq` dependencies into the image (Required by diesel)
# - Copy application files into the image
# ---------------------------------------------------
FROM gcr.io/distroless/cc-debian11 as runtime

# libpq related (required by diesel)
COPY --from=chef /usr/lib/*/libpq.so* /usr/lib/
COPY --from=chef /usr/lib/*/libgssapi_krb5.so* /usr/lib/
COPY --from=chef /usr/lib/*/libldap_r-2.4.so* /usr/lib/
COPY --from=chef /usr/lib/*/libkrb5.so* /usr/lib/
COPY --from=chef /usr/lib/*/libk5crypto.so* /usr/lib/
COPY --from=chef /usr/lib/*/libkrb5support.so* /usr/lib/
COPY --from=chef /usr/lib/*/liblber-2.4.so* /usr/lib/
COPY --from=chef /usr/lib/*/libsasl2.so* /usr/lib/
COPY --from=chef /usr/lib/*/libgnutls.so* /usr/lib/
COPY --from=chef /usr/lib/*/libp11-kit.so* /usr/lib/
COPY --from=chef /usr/lib/*/libidn2.so* /usr/lib/
COPY --from=chef /usr/lib/*/libunistring.so* /usr/lib/
COPY --from=chef /usr/lib/*/libtasn1.so* /usr/lib/
COPY --from=chef /usr/lib/*/libnettle.so* /usr/lib/
COPY --from=chef /usr/lib/*/libhogweed.so* /usr/lib/
COPY --from=chef /usr/lib/*/libgmp.so* /usr/lib/
COPY --from=chef /usr/lib/*/libffi.so* /usr/lib/
COPY --from=chef /lib/*/libcom_err.so* /lib/
COPY --from=chef /lib/*/libkeyutils.so* /lib/

# Application files
COPY --from=builder /server/target/release/server /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]
EXPOSE 3000