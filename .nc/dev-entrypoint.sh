#!/bin/sh
set -e

# The Nextcloud entrypoint only runs setup when $1 == "php-fpm".
# We stub out php-fpm with a no-op so it runs setup but doesn't block,
# then hand off to supervisord which manages the real php-fpm.

mkdir -p /tmp/fakebin
cat > /tmp/fakebin/php-fpm << 'EOF'
#!/bin/sh
exit 0
EOF
chmod +x /tmp/fakebin/php-fpm

PATH=/tmp/fakebin:$PATH /entrypoint.sh php-fpm || true

exec /usr/bin/supervisord -n -c /etc/supervisord.conf