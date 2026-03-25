#!/bin/bash
# apt-why installer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXTRAS=0

for arg in "$@"; do
    case $arg in
        --extras) EXTRAS=1 ;;
        --help|-h)
            echo "Usage: ./install.sh [--extras]"
            echo "  --extras   also install apt-pending (upgrade report tool)"
            exit 0 ;;
    esac
done

# Check dependencies
missing=()
command -v fzf &>/dev/null || missing+=("fzf")
command -v bc  &>/dev/null || missing+=("bc")

if [[ ${#missing[@]} -gt 0 ]]; then
    echo "Installing missing dependencies: ${missing[*]}"
    sudo apt-get install -y "${missing[@]}"
fi

sudo cp "$SCRIPT_DIR/apt-why" /usr/local/bin/apt-why
sudo chmod +x /usr/local/bin/apt-why
echo "Installed: apt-why"

if [[ $EXTRAS -eq 1 ]]; then
    sudo cp "$SCRIPT_DIR/apt-pending" /usr/local/bin/apt-pending
    sudo chmod +x /usr/local/bin/apt-pending
    echo "Installed: apt-pending"
fi
