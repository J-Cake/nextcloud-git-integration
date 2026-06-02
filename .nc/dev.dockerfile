# ============================================================
# Stage 1 — Build Caddy with caddy-git-jwt
# ============================================================
FROM golang:1.25-alpine AS caddy-builder

RUN apk add --no-cache git

RUN go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

# xcaddy pulls all of Caddy's deps + the plugin — expect ~2 min on first build.
# Docker layer cache means subsequent rebuilds are instant unless go.mod changes.
RUN xcaddy build \
    --with github.com/J-Cake/caddy-git-jwt \
    --output /usr/local/bin/caddy

# ============================================================
# Stage 2 — Nextcloud dev environment
# ============================================================
FROM nextcloud:32-fpm-alpine

# supervisor manages php-fpm + caddy in one container
RUN apk add --no-cache supervisor git composer

COPY --from=caddy-builder /usr/local/bin/caddy /usr/local/bin/caddy

COPY Caddyfile        /etc/caddy/Caddyfile
COPY supervisord.conf /etc/supervisord.conf
COPY dev-entrypoint.sh    /dev-entrypoint.sh
RUN chmod +x /dev-entrypoint.sh

# Install PHP dependencies for the gitviewer app
COPY composer.json composer.lock* /tmp/gitviewer/
RUN composer install --no-dev --optimize-autoloader --working-dir=/tmp/gitviewer

# Nextcloud auto-installs on first boot via the official entrypoint
ENV SQLITE_DATABASE=nextcloud \
    NEXTCLOUD_ADMIN_USER=admin \
    NEXTCLOUD_ADMIN_PASSWORD=admin \
    NEXTCLOUD_TRUSTED_DOMAINS="localhost 127.0.0.1"

EXPOSE 80 443 443/udp
VOLUME /var/www/html

ENTRYPOINT ["/dev-entrypoint.sh"]