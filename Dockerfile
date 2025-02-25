FROM lukemathwalker/cargo-chef:latest-rust-1.85.0 AS chef

WORKDIR /app

# Install the required system dependencies for our linking configuration
RUN apt-get update && apt-get install lld clang -y

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features native-tls,sqlite

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin nohead-rs_web

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install the required system dependencies for our linking configuration and litefs
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates fuse3 sqlite3 dos2unix \
  #  Cleanup
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Copy the workspace binaries from the builder stage to the runtime stage
COPY --from=builder /app/target/release/nohead-rs_web nohead-rs_web

# Copy sqlx binary from builder stage to runtime stage for migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
# Copy migrations directory
COPY ./db/migrations /app/db/migrations
# Copy the check_and_migrate.sh script to the runtime stage
COPY ./scripts/create_or_migrate_db.sh /usr/local/bin/create_or_migrate_db.sh
RUN dos2unix /usr/local/bin/create_or_migrate_db.sh && chmod +x /usr/local/bin/create_or_migrate_db.sh

# Setup sqlite3 on a separate volume
RUN mkdir -p /data
VOLUME /data

# Copy the litefs binary from the official image
COPY --from=flyio/litefs:main /usr/local/bin/litefs /usr/local/bin/litefs
COPY litefs.yml /etc/litefs.yml

# Copy the configuration file for runtime
COPY config/environments config/environments
COPY web/static web/static
ENV APP_ENVIRONMENT production

# Set the default environment variables for LiteFS
ENV LITEFS_DIR="/litefs"
ENV DATABASE_FILENAME="sqlite.db"
ENV DATABASE_PATH="${LITEFS_DIR}/${DATABASE_FILENAME}"
ENV DATABASE_URL="sqlite://${DATABASE_PATH}"

# LiteFS uses a proxy to forward requests to the internal port
ENV INTERNAL_PORT="8080"
ENV PORT="8081"

# Make SQLite CLI accessible via fly ssh console
# $ fly ssh console -C database-cli
RUN printf "#!/bin/sh\nset -x\nsqlite3 \$DATABASE_PATH" > /usr/local/bin/database-cli && chmod +x /usr/local/bin/database-cli

EXPOSE 8080
ENTRYPOINT ["litefs", "mount"]
