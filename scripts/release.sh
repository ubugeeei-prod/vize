#!/bin/bash
set -euo pipefail

# Usage: ./scripts/release.sh <bump> [-y]
# bump: patch | minor | major | alpha | beta | rc | release
# -y: Skip confirmation prompt
#
# Examples:
#   0.0.1-alpha  + patch   → 0.0.2-alpha
#   0.0.1-alpha  + minor   → 0.1.0-alpha
#   0.0.1-alpha  + major   → 1.0.0-alpha
#   0.0.1-alpha  + alpha   → 0.0.1-alpha.2
#   0.0.1-alpha  + beta    → 0.0.1-beta
#   0.0.1-beta   + rc      → 0.0.1-rc
#   0.0.1-rc     + release → 0.0.1

usage() {
  echo "Usage: $0 <bump> [-y]"
  echo "bump: patch | minor | major | alpha | beta | rc | release"
}

confirm_release() {
  if [ "${AUTO_CONFIRM:-}" = "-y" ]; then
    return 0
  fi

  local reply=""
  local prompt="Proceed with release? [y/N] "

  # Some task runners detach stdin while preserving the controlling terminal.
  if [ -t 2 ] && [ -r /dev/tty ] && [ -w /dev/tty ]; then
    printf "%s" "$prompt" > /dev/tty
    if ! IFS= read -r -n 1 reply < /dev/tty 2>/dev/null; then
      printf "\n" > /dev/tty
      echo "Error: Failed to read confirmation from tty. Re-run with -y to skip the prompt." >&2
      return 1
    fi
    printf "\n" > /dev/tty
  elif [ -t 0 ]; then
    printf "%s" "$prompt" >&2
    if ! IFS= read -r -n 1 reply; then
      echo >&2
      echo "Error: Failed to read confirmation from stdin. Re-run with -y to skip the prompt." >&2
      return 1
    fi
    echo
  else
    echo "Error: Confirmation requires an interactive terminal. Re-run with -y to skip the prompt." >&2
    return 1
  fi

  if [[ ! "$reply" =~ ^[Yy]$ ]]; then
    echo "Aborted."
    return 1
  fi
}

main() {
  local bump auto_confirm current_version
  local major minor patch prerelease prerelease_num

  bump=${1-}
  auto_confirm=${2-}

  if [ -z "$bump" ]; then
    usage
    exit 1
  fi

  BUMP="$bump"
  AUTO_CONFIRM="$auto_confirm"

  # Get current version from Cargo.toml
  current_version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
  CURRENT_VERSION="$current_version"
  echo "Current version: $CURRENT_VERSION"

  # Parse version components
  if [[ "$CURRENT_VERSION" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)(-([a-zA-Z]+)(\.([0-9]+))?)?$ ]]; then
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    patch="${BASH_REMATCH[3]}"
    prerelease="${BASH_REMATCH[5]}"
    prerelease_num="${BASH_REMATCH[7]}"
  else
    echo "Error: Cannot parse version: $CURRENT_VERSION"
    exit 1
  fi

  # Calculate new version based on bump type
  case "$BUMP" in
    patch)
      patch=$((patch + 1))
      if [ -n "$prerelease" ]; then
        NEW_VERSION="$major.$minor.$patch-$prerelease"
      else
        NEW_VERSION="$major.$minor.$patch"
      fi
      ;;
    minor)
      minor=$((minor + 1))
      patch=0
      if [ -n "$prerelease" ]; then
        NEW_VERSION="$major.$minor.$patch-$prerelease"
      else
        NEW_VERSION="$major.$minor.$patch"
      fi
      ;;
    major)
      major=$((major + 1))
      minor=0
      patch=0
      if [ -n "$prerelease" ]; then
        NEW_VERSION="$major.$minor.$patch-$prerelease"
      else
        NEW_VERSION="$major.$minor.$patch"
      fi
      ;;
    alpha)
      if [ "$prerelease" = "alpha" ]; then
        if [ -n "$prerelease_num" ]; then
          prerelease_num=$((prerelease_num + 1))
        else
          prerelease_num=2
        fi
        NEW_VERSION="$major.$minor.$patch-alpha.$prerelease_num"
      else
        NEW_VERSION="$major.$minor.$patch-alpha"
      fi
      ;;
    beta)
      NEW_VERSION="$major.$minor.$patch-beta"
      ;;
    rc)
      NEW_VERSION="$major.$minor.$patch-rc"
      ;;
    release)
      NEW_VERSION="$major.$minor.$patch"
      ;;
    *)
      echo "Error: Unknown bump type: $BUMP"
      echo "Valid options: patch | minor | major | alpha | beta | rc | release"
      exit 1
      ;;
  esac

  TAG="v$NEW_VERSION"

  echo "New version: $NEW_VERSION (tag: $TAG)"
  echo ""

  confirm_release

  # Check for uncommitted changes
  if [ -n "$(git status --porcelain)" ]; then
    echo "Error: There are uncommitted changes. Please commit or stash them first."
    exit 1
  fi

  # Check if tag already exists
  if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag $TAG already exists."
    exit 1
  fi

  # Update Cargo.toml (workspace version and dependencies)
  echo "Updating Cargo.toml..."
  sed -i.bak 's/^version = ".*"/version = "'"$NEW_VERSION"'"/' Cargo.toml
  # Update internal crate versions in workspace.dependencies
  sed -i.bak 's/version = "'"$CURRENT_VERSION"'"/version = "'"$NEW_VERSION"'"/g' Cargo.toml
  rm -f Cargo.toml.bak

  # Update Cargo.lock to reflect the new versions
  echo "Updating Cargo.lock..."
  cargo update --workspace

  # Update npm package versions
  echo "Updating npm packages..."
  for pkg in npm/*/; do
    pkg=${pkg%/}  # remove trailing slash
    if [ -f "$pkg/package.json" ]; then
      node -e "
        const fs = require('fs');
        const pkg = JSON.parse(fs.readFileSync('$pkg/package.json', 'utf8'));
        pkg.version = '$NEW_VERSION';
        // NOTE: @vizejs/native-* optionalDependencies are NOT bumped here because they
        // don't exist on npm until the Release workflow publishes them. Bumping them
        // would break pnpm-lock.yaml since the new version can't be resolved.
        // The release workflow injects the correct version at publish time.
        fs.writeFileSync('$pkg/package.json', JSON.stringify(pkg, null, 2) + '\n');
      "
      echo "  Updated $pkg/package.json"
    fi
  done

  # Update version references in READMEs
  echo "Updating READMEs..."
  find npm -name 'README.md' -exec sed -i.bak "s/$CURRENT_VERSION/$NEW_VERSION/g" {} \;
  find npm -name 'README.md.bak' -delete
  sed -i.bak "s/$CURRENT_VERSION/$NEW_VERSION/g" README.md
  rm -f README.md.bak

  # Commit changes
  echo "Committing changes..."
  git add Cargo.toml Cargo.lock npm/*/package.json README.md
  # Add READMEs that are tracked (some may be gitignored)
  git add npm/*/README.md 2>/dev/null || true
  git commit -m "chore: release v$NEW_VERSION"

  # Create tag
  echo "Creating tag $TAG..."
  git tag -a "$TAG" -m "Release $NEW_VERSION"

  # Push to remote
  echo "Pushing to remote..."
  git push origin main
  git push origin "$TAG"

  echo ""
  echo "Release $NEW_VERSION complete!"
  echo "GitHub Actions will now publish to npm and crates.io."
  echo "Check: https://github.com/ubugeeei/vize/actions"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
