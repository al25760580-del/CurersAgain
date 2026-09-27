#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec cargo run --bin title_reference -- "${1:-../asset-lab}"
