# apt-why

Two tools for understanding your Debian/Ubuntu packages.

## Install

```bash
git clone git@gitlab.com:safteinzz/apt-why.git
cd apt-why
./install.sh            # installs apt-why only
./install.sh --extras   # also installs apt-pending
```

Installs `fzf` and `bc` if missing.

---

## apt-why — interactive investigator

```bash
apt-why                  # browse all installed packages
apt-why --upgradable     # browse packages with pending upgrades
```

Fuzzy-find any package. Press `enter` to open its dossier:

- `[M]` manual / `[A]` auto-installed, install date
- Upgrade version if available
- What needs it, what it depends on — navigable via a second fuzzy finder
- Packages installed in the same session (context clue)

---

## apt-pending — upgrade report `(--extras)`

Run after `sudo apt update` to see what's waiting and why.

```bash
apt-pending              # full report
apt-pending --quick      # one line per package: size + reason
apt-pending --auto       # auto-installed grouped by cause
apt-pending --kernel     # kernel/firmware/microcode only
apt-pending --apps       # your manually installed packages only
apt-pending --sizes      # top 20 by disk size
```

Scriptable and pipeable:

```bash
apt-pending --quick | grep clang
apt-pending --quick > pre-upgrade-$(date +%F).txt
```

---

## Requirements

`fzf`, `bc` — installed automatically by `install.sh`
