#!/usr/bin/env bash
RED='\033[0;31m'
NC='\033[0m'

echo -e "You're deploying reflective to ${RED}production!${NC}"

## Check that there are no uncommited changes
if [ -n "$(git status --porcelain)" ]; then
  echo "There are uncomitted changes"
  exit 1
else
  echo "No uncomitted changes"
fi

## Print current version
cd service
version=$(cargo metadata --format-version=1 --no-deps | jq '.packages[0].version')
cd ..

echo "Latest version: ${version}"

## Prompt for next version (default current version)
read -p "Enter version to be deployed: " newVersion 

## Write version to Cargo.toml and package.json
grep -q "version = ${version}" service/Cargo.toml
if [[ $? -ne 0 ]]; then
  echo "service is not at version ${version}" 
  exit 1
fi

grep -q "\"version\": ${version}" ui/package.json
if [[ $? -ne 0 ]]; then
  echo "ui is not at version ${version}" 
  exit 1
fi

echo "Writing new version to Cargo.toml and package.json"
cd service
cargo set-version "${newVersion}"
cd ..
cd ui
npm version "${newVersion}"
cd ..

## Build docker image
echo "Building docker image"
docker buildx build --platform=linux/amd64 -t "ghcr.io/beingflo/reflective:${newVersion}" .
if [[ $? -ne 0 ]]; then
  echo "docker build failed" 
  exit 1
fi

## Push docker image
echo "Pushing docker image"
docker push "ghcr.io/beingflo/reflective:${newVersion}"
if [[ $? -ne 0 ]]; then
  echo "docker push failed" 
  exit 1
fi

## Write new image tag in docker compose file
cleanVersion=$(echo "${version}" | tr -d '"')
grep -q "image: \"ghcr.io/beingflo/reflective:${cleanVersion}\"" ./docker-compose.prod.yml
if [[ $? -ne 0 ]]; then
  echo "docker compose is not at version ${version}" 
  exit 1
fi

echo "Write new version to docker compose file"
sed -i '' -e "s/image: \"ghcr.io\/beingflo\/reflective:${cleanVersion}\"/image: \"ghcr.io\/beingflo\/reflective:${newVersion}\"/" ./docker-compose.prod.yml

## Upgrade image running on omni
echo "Upgrading reflective on omni"
docker --context omni compose --file docker-compose.prod.yml pull
if [[ $? -ne 0 ]]; then
  echo "docker compose pull failed" 
  exit 1
fi

docker --context omni compose --file docker-compose.prod.yml up -d
if [[ $? -ne 0 ]]; then
  echo "docker compose up failed" 
  exit 1
fi

## Commit and tag version
echo "Commit and tag release"
git commit -am "Release ${newVersion}"
git tag "${newVersion}"
git push origin --tags
