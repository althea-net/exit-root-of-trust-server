#!/bin/bash
set -eux
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
pushd backend
cross build --target x86_64-unknown-linux-musl --release
cp target/x86_64-unknown-linux-musl/release/exit_trust_root_server ../scripts/
popd
pushd $DIR
    ansible-playbook -i hosts deploy.yml
popd