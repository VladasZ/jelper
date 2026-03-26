#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"
MACOS_BINARY="$2"
TAP_KEY="$3"

SHA256=$(sha256sum "$MACOS_BINARY" | awk '{print $1}')

mkdir -p ~/.ssh
echo "$TAP_KEY" > ~/.ssh/tap_key
chmod 600 ~/.ssh/tap_key
ssh-keyscan github.com >> ~/.ssh/known_hosts

GIT_SSH_COMMAND="ssh -i ~/.ssh/tap_key" git clone git@github.com:VladasZ/homebrew-tap.git

sed -i "s|releases/download/.*/jelper-macos|releases/download/${VERSION}/jelper-macos|" homebrew-tap/Formula/jelper.rb
sed -i "s/version \".*\"/version \"${VERSION#v}\"/" homebrew-tap/Formula/jelper.rb
sed -i "s/sha256 \".*\"/sha256 \"${SHA256}\"/" homebrew-tap/Formula/jelper.rb

cd homebrew-tap
git config user.email "actions@github.com"
git config user.name "GitHub Actions"
git add Formula/jelper.rb
git commit -m "jelper ${VERSION}"
GIT_SSH_COMMAND="ssh -i ~/.ssh/tap_key" git push
