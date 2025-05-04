#!/usr/bin/env bash
die() { echo "$*" 1>&2 ; exit 1; }

docker --context arm compose --file compose.prod.yml pull || die "Failed to pull new image"
docker --context arm compose --file compose.prod.yml up -d || die "Failed to bring compose up"