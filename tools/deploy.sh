#!/bin/sh
# Build the Bevy game on fridge and deploy Lucky Monkey (runs on fridge, go.fridge.run/luckymonkey).
#
#   tools/deploy.sh          build + deploy
#   tools/deploy.sh build    build only (fridge:~/build/luckymonkey/public/game)
#   tools/deploy.sh data     regenerate world.json, push game/data/*.json live (seconds; no Rust build, no restart)
#
# Freezer has too little disk for a Rust/Bevy build, so the build happens on
# fridge as user `fridge` (rustup + wasm-bindgen-cli in ~fridge/.cargo).
set -eu
cd "$(dirname "$0")/.."
HOST=fridge@fridge
B=build/luckymonkey

python3 tools/gen-world.py   # islands.json + nodes.json -> world.json
validate() { for f in game/data/*.json; do python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$f" || { echo "invalid JSON: $f"; exit 1; }; done; }
validate
if [ "${1:-}" = data ]; then
  ssh "$HOST" "mkdir -p $B/public/game/data"
  rsync -a game/data/ "$HOST:$B/public/game/data/"
  ssh "$HOST" "set -e; export SUDO_ASKPASS=/home/fridge/.sudo-askpass
    sudo -A rsync -a $B/public/game/data/ /srv/apps/luckymonkey/public/game/data/
    sudo -A chown -R root:root /srv/apps/luckymonkey/public/game/data && sudo -A chmod -R a+rX /srv/apps/luckymonkey/public/game/data"
  echo "data live: reload the page"
  exit 0
fi
python3 tools/make-textures.py
python3 tools/wiki-source.py   # docs/source/: the code as wiki pages (commit it if it changed)
python3 tools/gen-docs.py      # docs/failures.md + nodes.md from the game data
rsync -a --delete --exclude node_modules --exclude .next --exclude .git --exclude game/target --exclude public/game \
  --exclude tsconfig.tsbuildinfo ./ "$HOST:$B/"
ssh "$HOST" "set -e; cd $B/game; export PATH=\$HOME/.cargo/bin:\$PATH
  cargo build --release --target wasm32-unknown-unknown
  rm -rf ../public/game
  wasm-bindgen --target web --no-typescript --out-dir ../public/game target/wasm32-unknown-unknown/release/luckymonkey.wasm
  mkdir -p ../public/game/data && cp data/*.json ../public/game/data/
  ls -la ../public/game"
[ "${1:-}" = build ] && exit 0

ssh "$HOST" "set -e; export SUDO_ASKPASS=/home/fridge/.sudo-askpass
  sudo -A rsync -a --delete --exclude .next --exclude node_modules --exclude game --exclude public/game $B/ /srv/apps/luckymonkey/
  sudo -A rsync -a --delete $B/public/game/ /srv/apps/luckymonkey/public/game/
  sudo -A chown -R root:root /srv/apps/luckymonkey && sudo -A chmod -R a+rX /srv/apps/luckymonkey
  sudo -A fridge-next luckymonkey build >/tmp/lm-build.log 2>&1 || { tail -30 /tmp/lm-build.log; exit 1; }
  sudo -A fridge-next luckymonkey restart
  sleep 3; sudo -A docker ps --filter name=luckymonkey --format '{{.Names}} {{.Status}}'"
