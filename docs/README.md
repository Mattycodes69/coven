# Coven Documentation

This directory contains the public Coven docs app and the broader repo-local documentation corpus.

The public site is intentionally curated by `docs.json`. Do not expose every markdown file automatically: this repo also contains maintainer notes, planning docs, and scaffolded topic files that are not ready for the public app.

## Public App Shape

Current public tabs:

- `Start` - overview, install path, TUI, troubleshooting, concepts, roadmap.
- `Runtime` - architecture, operational model, safety/auth posture, sessions, harness adapters.
- `Integrate` - local API contract, client integration, and comux demo-loop contract.
- `Reference` - current CLI/API lookup, glossary, and product spec.

Excluded from the public app:

- generated `docs/docs/**` scaffold pages;
- stub pages containing `Stub - fill in`;
- future-only orchestration command guides;
- internal maintenance, verification, and implementation planning notes.

## Local Build

Use the repo's working Node runtime. On this machine, prefer nvm Node if Homebrew Node is broken:

```sh
PATH=/Users/buns/.nvm/versions/node/v24.13.0/bin:$PATH npm install
PATH=/Users/buns/.nvm/versions/node/v24.13.0/bin:$PATH npm run docs:check
```

The build writes static output to `dist/docs-site/`.

## Editing Rules

When adding a public page:

1. Write or update the markdown file.
2. Add it to `docs.json`.
3. Make sure it is not a scaffold stub.
4. Run `npm run docs:check`.
5. Run from repo root:

```sh
python scripts/check-secrets.py
git diff --check
```

Use stable, verified product language. Do not document future commands or endpoints as if they exist.
