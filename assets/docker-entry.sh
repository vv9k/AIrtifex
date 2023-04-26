#!/bin/sh

echo "Starting nginx"
/usr/sbin/nginx -g "daemon on; master_process on;"

/usr/local/bin/airtifex-api -c /etc/airtifex/config.yaml serve