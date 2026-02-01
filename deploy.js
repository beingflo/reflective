#!/usr/bin/env bun
import { $ } from "bun";
import { unlinkSync } from "fs";
import { parseArgs } from "util";

// Exit on error and ensure decrypted .env file is cleaned up
$.throws(true);
process.on("exit", () => {
  try {
    unlinkSync(".env.prod");
  } catch {}
});

// Parse CLI arguments
const { values } = parseArgs({
  args: Bun.argv.slice(2),
  options: {
    build: { type: "boolean", short: "b", default: false },
    deploy: { type: "boolean", short: "d", default: false },
  },
  strict: true,
});

// If no flags specified, do both
const shouldBuild = values.build || (!values.build && !values.deploy);
const shouldDeploy = values.deploy || (!values.build && !values.deploy);

console.log("Deploying reflective to production!");

if (shouldBuild) {
  // Check for uncommitted changes
  const status = await $`git status --porcelain`.text();
  if (status.trim()) {
    console.error("There are uncommitted changes");
    //process.exit(1);
  }

  // Get current version from Cargo.toml
  const metadata = await $`cargo metadata --format-version=1 --no-deps`
    .cwd("service")
    .json();
  const version = metadata.packages[0].version;
  console.log(`Latest version: ${version}`);

  // Calculate next version
  const versionParts = version.split(".");
  versionParts[versionParts.length - 1] = String(
    Number(versionParts[versionParts.length - 1]) + 1,
  );
  const nextVersion = versionParts.join(".");

  // Prompt for version
  const newVersion =
    prompt(`Enter version to be deployed [${nextVersion}]:`)?.trim() ||
    nextVersion;

  // Set new version in Cargo.toml
  await $`cargo set-version ${newVersion}`.cwd("service");

  // Build Docker image
  const imageName = `ghcr.io/beingflo/reflective:${newVersion}`;
  await $`docker buildx build -t ${imageName} .`;
  await $`docker push ${imageName}`;

  // Update compose file
  const composeFile = await Bun.file("./compose.prod.yml").text();
  const oldImageLine = `image: "ghcr.io/beingflo/reflective:${version}"`;
  const newImageLine = `image: "ghcr.io/beingflo/reflective:${newVersion}"`;
  const updatedCompose = composeFile.replace(oldImageLine, newImageLine);
  await Bun.write("./compose.prod.yml", updatedCompose);

  // Git operations
  await $`git commit -am "Release ${newVersion}"`;
  await $`git tag ${newVersion}`;
  await $`git push`;
  await $`git push origin --tags`;
}

if (shouldDeploy) {
  // Get age key from 1Password
  const ageKey =
    await $`op item get "SOPS age key - reflective" --reveal --fields "private key"`.text();

  // Decrypt .env file
  await $`sops -d --input-type dotenv --output-type dotenv .env.prod.enc > .env.prod`.env(
    {
      ...process.env,
      SOPS_AGE_KEY: ageKey.trim(),
    },
  );

  // Pull and deploy
  await $`docker --context arm compose --file compose.prod.yml pull`;
  await $`docker --context arm compose --file compose.prod.yml up --build -d`;
}
