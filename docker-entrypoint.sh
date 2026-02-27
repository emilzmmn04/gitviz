#!/bin/sh
set -e
envsubst '${PORT}' < /etc/nginx/conf.d/default.conf > /tmp/default.conf
cp /tmp/default.conf /etc/nginx/conf.d/default.conf
exec nginx -g 'daemon off;'
