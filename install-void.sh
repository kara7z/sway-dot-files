#!/usr/bin/env bash
# Explicit Void Linux entry point; all standard installer options are accepted.
set -euo pipefail
exec bash "$(dirname "$0")/install.sh" --profile void "$@"
