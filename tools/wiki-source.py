#!/usr/bin/env python3
"""Generate docs/source/*.md: the game's source code as browsable wiki pages.

Run from the repo root (tools/deploy.sh does it). Every Rust file becomes a page
with one heading per top-level item (`fn route`, `struct Monkey`, `impl Net`...),
so the articles can link straight to code: [route()](source/ops.md#fn_route).
The fragment is the heading text with spaces as underscores (DokuWiki section ids).
Other files (build scripts, host page, container) become one page each.

Why generated pages and not GitHub links: the repo is private, and the wiki is
where readers are; the pages are rebuilt from the code on every deploy, so they
never drift.
"""
import os, re, subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "docs", "source")
GITHUB = "https://github.com/LuckyMonkey/luckymonkey/blob/main/"

RUST = [
    ("main.rs", "main", "App setup: the window and canvas, shared fonts and textures (`Art`), procedural textures, the random generator, screens.", "engine.md"),
    ("link.rs", "link", "Deep links: reads the URL hash (`#node=…&incident=…`) so a link can open the game at a place with something staged.", "interface.md"),
    ("world.rs", "world", "The map: loading world.json, the pyramid and bank fixes, the terrain meshes, edge islands, the void, decorations, node cards.", "world.md"),
    ("water.rs", "water", "Living water: waterfall streams, lagoon bubbles, glints, and data frames floating down the rivers.", "world.md"),
    ("npc.rs", "npc", "Wandering critters: monkeys, ghosts, bananas, lizards, crocodiles.", "world.md"),
    ("clouds.rs", "clouds", "Clouds drawn like the original divs; rain; storms that flash and send lightning strikes.", "world.md"),
    ("view.rs", "view", "The camera: pan, zoom, touch, picking nodes, following, deep-link zoom.", "interface.md"),
    ("ui.rs", "ui", "Welcome screen, zoom buttons, hint bar, the node panel.", "interface.md"),
    ("sim.rs", "sim", "The rules, with no Bevy: faults, responses, the incident director, crew, routing, health and SLA, scoring, the event log. Tested natively by game/simtest.", "simulation.md"),
    ("data.rs", "data", "Live JSON: fetches tuning/world/npcs from the page, deep-merges tuning over the built-in copy, falls back on errors.", "build-deploy.md"),
    ("ops.rs", "ops", "The game screen: steps the sim, HUD, Time to Loss, crew roster, alerts, log, map overlays (rope links, badges), edge pins, tour, summary, keys and clicks into orders.", "gameplay.md"),
]
OTHER = [
    ("game/Cargo.toml", "cargo-toml", "toml", "The Rust crate: Bevy features, dependencies, the size-optimised release profile.", "build-deploy.md"),
    ("tools/deploy.sh", "deploy-sh", "bash", "Build on fridge and deploy.", "build-deploy.md"),
    ("tools/gen-world.py", "gen-world", "python", "Builds world.json from islands.json + nodes.json: the archipelago, the river, hills, islets, vines.", "world.md"),
    ("tools/gen-docs.py", "gen-docs", "python", "Writes the Failure types and Nodes and lore pages from the game data.", "build-deploy.md"),
    ("tools/export-world.cjs", "export-world", "javascript", "(Legacy) ran the TypeScript map generators; replaced by gen-world.py.", "world.md"),
    ("tools/make-textures.py", "make-textures", "python", "Packs the tile sprites into the atlas; draws the node card textures.", "world.md"),
    ("tools/subset-fonts.py", "subset-fonts", "python", "Re-subsets the embedded fonts to the characters in use.", "extending.md"),
    ("tools/browser-test.sh", "browser-test", "bash", "Plays the game in headless Firefox and saves screenshots.", "build-deploy.md"),
    ("tools/wiki-source.py", "wiki-source", "python", "This generator.", "index.md"),
    ("app/page.tsx", "page-tsx", "tsx", "The host page: one canvas, a loader, and the dynamic import of the game.", "overview.md"),
    ("compose.yml", "compose", "yaml", "The container on the shared fridge-next runtime.", "build-deploy.md"),
    ("next.config.mjs", "next-config", "javascript", "Next.js settings: the /luckymonkey base path.", "build-deploy.md"),
    ("lib/world-data.ts", "world-data", "typescript", "THE MAP SOURCE: nodes, rivers, terrain and decoration generators.", "world.md"),
    ("lib/npc-data.ts", "npc-data", "typescript", "The critter roster.", "world.md"),
]


def anchor(heading):
    """DokuWiki section id for a heading (good enough for our ASCII headings)."""
    return re.sub(r"[^a-z0-9_]+", "_", heading.lower()).strip("_")


def scan_rust(src):
    """Split Rust source into (kind, name, doc, code) top-level items by brace depth."""
    items, i, n = [], 0, len(src)
    depth, start, pending = 0, 0, None

    def skip_string(j):
        # j at the opening quote; handles escapes.
        j += 1
        while j < n and src[j] != '"':
            j += 2 if src[j] == "\\" else 1
        return j + 1

    item_start = 0
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            i = src.find("\n", i)
            i = n if i < 0 else i
            continue
        if src.startswith("/*", i):
            i = src.find("*/", i) + 2
            continue
        if c == 'r' and re.match(r'r#*"', src[i:i + 4]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == '_')):
            hashes = re.match(r'r(#*)"', src[i:]).group(1)
            i = src.find('"' + hashes, i + len(hashes) + 2) + len(hashes) + 1
            continue
        if c == '"':
            i = skip_string(i)
            continue
        if c == "'":
            m = re.match(r"'(\\.|[^\\'])'", src[i:i + 4]) or re.match(r"'\\u\{[0-9a-fA-F]+\}'", src[i:i + 12])
            if m:
                i += len(m.group(0))
                continue
        if c in "{[(":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                end = src.find("\n", i)
                end = n if end < 0 else end
                items.append(src[item_start:end].strip("\n"))
                item_start = end + 1
        elif c == ";" and depth == 0:
            end = src.find("\n", i)
            end = n if end < 0 else end
            items.append(src[item_start:end].strip("\n"))
            item_start = end + 1
        i += 1
    tail = src[item_start:].strip()
    if tail:
        items.append(tail)
    return items


def describe(block):
    """(kind, name, doc lines, code) for one top-level block."""
    lines = block.split("\n")
    doc, code_start = [], 0
    for k, ln in enumerate(lines):
        s = ln.strip()
        if s.startswith("//!"):
            continue
        if s.startswith("///"):
            doc.append(s[3:].strip())
        elif s == "" and not doc:
            continue
        else:
            code_start = k
            break
    code = "\n".join(lines[code_start:]).strip("\n")
    body = re.sub(r"^\s*#\[[^\]]*\]\s*", "", code, flags=re.M)
    m = re.search(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(fn|struct|enum|const|static|type|mod|trait|use)\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M)
    if re.match(r"^\s*(?:pub\s+)?use\s", body):
        return "use", "", doc, code
    if re.match(r"^\s*impl", body):
        m2 = re.match(r"^\s*impl(?:<[^>]*>)?\s+([^{]+?)\s*\{", body, re.S)
        return "impl", (m2.group(1).strip() if m2 else "block"), doc, code
    if m:
        return m.group(1), m.group(2), doc, code
    return "item", "", doc, code


def module_doc(src):
    return "\n".join(ln.strip()[3:].strip() for ln in src.split("\n") if ln.strip().startswith("//!"))


def rust_page(fname, slug, blurb, article):
    path = os.path.join(ROOT, "game", "src", fname)
    src = open(path, encoding="utf-8").read()
    blocks = [describe(b) for b in scan_rust(src)]
    out = [
        "---", f"title: Lucky Monkey source - {fname}", "---", "",
        f"# Source: game/src/{fname}", "",
        f"**{blurb}** Explained in [{article.replace('.md', '').replace('-', ' ')}](../{article}). "
        f"{src.count(chr(10))} lines. [On GitHub]({GITHUB}game/src/{fname}) (needs repo access). "
        "This page is generated from the code on every deploy by `tools/wiki-source.py`; "
        "back to the [source index](index.md).", "",
    ]
    md = module_doc(src)
    if md:
        out += ["## What this file says about itself", "", md, ""]
    toc, sections, uses = [], [], []
    for kind, name, doc, code in blocks:
        if not code.strip() or code.strip().startswith("#!["):
            continue
        if kind == "use":
            uses.append(code)
            continue
        heading = f"{kind} {name}".strip()
        first = doc[0] if doc else ""
        toc.append(f"| [{heading}](#{anchor(heading)}) | {first} |")
        sec = [f"### {heading}", ""]
        if doc:
            sec += [" ".join(doc), ""]
        sec += ["```rust", code, "```", ""]
        sections += sec
    out += ["## Contents", "", "| Item | What it is |", "|---|---|"] + toc + [""]
    if uses:
        out += ["## Imports", "", "```rust", "\n".join(uses), "```", ""]
    out += ["## Code", ""] + sections
    open(os.path.join(OUT, f"{slug}.md"), "w", encoding="utf-8").write("\n".join(out))
    return slug, fname, blurb, [(b[0], b[1]) for b in blocks if b[0] != "use"]


def other_page(rel, slug, lang, blurb, article):
    src = open(os.path.join(ROOT, rel), encoding="utf-8").read()
    out = [
        "---", f"title: Lucky Monkey source - {os.path.basename(rel)}", "---", "",
        f"# Source: {rel}", "",
        f"**{blurb}** Explained in [{article.replace('.md', '').replace('-', ' ')}](../{article}). "
        f"[On GitHub]({GITHUB}{rel}) (needs repo access). Generated by `tools/wiki-source.py`; "
        "back to the [source index](index.md).", "",
        f"```{lang}", src.rstrip("\n"), "```", "",
    ]
    open(os.path.join(OUT, f"{slug}.md"), "w", encoding="utf-8").write("\n".join(out))


def main():
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT):
        if f.endswith(".md"):
            os.remove(os.path.join(OUT, f))
    rows = []
    for fname, slug, blurb, article in RUST:
        slug, fname, blurb, items = rust_page(fname, slug, blurb, article)
        fns = sum(1 for k, _ in items if k == "fn")
        rows.append(f"| [game/src/{fname}]({slug}.md) | {blurb} | {fns} functions, {len(items)} items |")
    others = []
    for rel, slug, lang, blurb, article in OTHER:
        other_page(rel, slug, lang, blurb, article)
        others.append(f"| [{rel}]({slug}.md) | {blurb} |")
    try:
        rev = subprocess.run(["git", "-C", ROOT, "log", "-1", "--format=%h %ad", "--date=short"], capture_output=True, text=True).stdout.strip()
    except OSError:
        rev = ""
    index = [
        "---", "title: Lucky Monkey source code", "---", "",
        "# Source code", "",
        "The whole game, readable in the wiki. Each Rust file has a heading for every "
        "function, struct and impl, so the articles link straight to the lines they talk "
        f"about. Generated from the code by `tools/wiki-source.py` (last: {rev}). "
        "Back to the [documentation](../index.md).", "",
        "**Where to start reading:** [main.rs](main.md) (how the app is put together), then "
        "[ops.rs](ops.md) (the game), starting at [the plugin](ops.md#fn_plugin) and "
        "the rules in [sim.rs](sim.md#impl_sim).", "",
        "## The game (Rust, compiled to WebAssembly)", "",
        "| File | What it does | Size |", "|---|---|---|",
    ] + rows + [
        "", "## Around it: data, build, hosting", "",
        "| File | What it is |", "|---|---|",
    ] + others + [""]
    open(os.path.join(OUT, "index.md"), "w", encoding="utf-8").write("\n".join(index))
    print(f"wrote {len(RUST) + len(OTHER) + 1} pages to docs/source/")


if __name__ == "__main__":
    main()
