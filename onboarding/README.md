# Command Deck onboarding pack

This directory contains a lab onboarding content pack for Command Deck.

It is intentionally plain files:

- `templates/` contains Command Deck TOML tabs. One TOML file becomes one tab.
- `guides/` contains Markdown guides linked from the cards.

## Use without changing the app

Open Command Deck settings and point:

```text
Templates dir: <repo>/onboarding/templates
Guides dir:    <repo>/onboarding/guides
```

That is enough for the current app. The app already loads one TOML file per tab and looks up guide files by filename in the configured guides directory.

## When app changes would be useful

No app change is required to use these files manually. App changes would only be needed if you want:

1. Auto-seeding into `~/.config/command-deck/` on first run.
2. True guide-only cards without copy/dry-run/execute buttons.
3. Search across cards and guides.
4. A persistent onboarding checklist.

For now, this pack works as a drop-in content directory.
