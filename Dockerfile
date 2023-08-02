# ---------------------------------------------------
# 1 - Build Stage
#
# Use official rust image to for application build
# ---------------------------------------------------
FROM rust:latest as build

# Setup working directory
WORKDIR /usr/src/server
COPY . .

# Install dependency (Required by diesel)
RUN apt-get update && apt-get install libpq5 -y

# Build application
RUN cargo install --path .

# ---------------------------------------------------
# 2 - Deploy Stage
#
# Use a distroless image for minimal container size
# - Copy `libpq` dependencies into the image (Required by diesel)
# - Copy application files into the image
# ---------------------------------------------------
FROM gcr.io/distroless/cc-debian11

# libpq related (required by diesel)
COPY --from=build /usr/lib/*/libpq.so* /usr/lib/
COPY --from=build /usr/lib/*/libgssapi_krb5.so* /usr/lib/
COPY --from=build /usr/lib/*/libldap_r-2.4.so* /usr/lib/
COPY --from=build /usr/lib/*/libkrb5.so* /usr/lib/
COPY --from=build /usr/lib/*/libk5crypto.so* /usr/lib/
COPY --from=build /usr/lib/*/libkrb5support.so* /usr/lib/
COPY --from=build /usr/lib/*/liblber-2.4.so* /usr/lib/
COPY --from=build /usr/lib/*/libsasl2.so* /usr/lib/
COPY --from=build /usr/lib/*/libgnutls.so* /usr/lib/
COPY --from=build /usr/lib/*/libp11-kit.so* /usr/lib/
COPY --from=build /usr/lib/*/libidn2.so* /usr/lib/
COPY --from=build /usr/lib/*/libunistring.so* /usr/lib/
COPY --from=build /usr/lib/*/libtasn1.so* /usr/lib/
COPY --from=build /usr/lib/*/libnettle.so* /usr/lib/
COPY --from=build /usr/lib/*/libhogweed.so* /usr/lib/
COPY --from=build /usr/lib/*/libgmp.so* /usr/lib/
COPY --from=build /usr/lib/*/libffi.so* /usr/lib/
COPY --from=build /lib/*/libcom_err.so* /lib/
COPY --from=build /lib/*/libkeyutils.so* /lib/

# Application files
COPY --from=build /usr/local/cargo/bin/server /usr/local/bin/server

ENTRYPOINT ["server"]
EXPOSE 3000