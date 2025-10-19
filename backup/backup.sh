#! /bin/bash
die() { echo "$*" 1>&2 ; exit 1; }

set -eu
set -o pipefail

[ -z "$S3_ACCESS_KEY_ID" ] && die "S3_ACCESS_KEY_ID missing"
[ -z "$S3_SECRET_ACCESS_KEY" ] && die "S3_SECRET_ACCESS_KEY missing"
[ -z "$S3_BUCKET" ] && die "S3_BUCKET missing"
[ -z "$POSTGRES_DATABASE" ] && die "POSTGRES_DATABASE missing"
[ -z "$S3_PREFIX" ] && die "S3_PREFIX missing"
[ -z "$POSTGRES_HOST" ] && die "POSTGRES_HOST missing"
[ -z "$POSTGRES_USER" ] && die "POSTGRES_USER missing"
[ -z "$S3_ENDPOINT" ] && die "S3_ENDPOINT missing"

# Needed by aws_cli
export AWS_ACCESS_KEY_ID=$S3_ACCESS_KEY_ID
export AWS_SECRET_ACCESS_KEY=$S3_SECRET_ACCESS_KEY
export AWS_DEFAULT_REGION=$S3_REGION
  
# Needed by pg_dump
export PGPASSWORD=$POSTGRES_PASSWORD

aws_args="--endpoint-url "$S3_ENDPOINT/$S3_BUCKET""

echo "Creating backup of $POSTGRES_DATABASE database..."

START=$(date +%s)
pg_dump --format=custom \
        -h $POSTGRES_HOST \
        -p $POSTGRES_PORT \
        -U $POSTGRES_USER \
        -d $POSTGRES_DATABASE \
        > db.dump
SIZE=$(ls -nlt db.dump | head -n1 | awk '{print $5}')

s3_uri_base="s3://${S3_PREFIX}/$(date +%Y-%m-%d_%H-%M)/${POSTGRES_DATABASE}.dump"

local_file="db.dump"
s3_uri="$s3_uri_base"

echo "Uploading backup to $S3_BUCKET..."
aws $aws_args s3 cp "$local_file" "$s3_uri"
rm "$local_file"

END=$(date +%s)

echo "Backup complete."
echo "Start: ${START}, END: ${END}, SIZE: ${SIZE}"
