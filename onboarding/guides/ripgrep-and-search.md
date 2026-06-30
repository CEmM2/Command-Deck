# Ripgrep and search

`ripgrep`, usually run as `rg`, is a fast recursive text search tool.

## Basic use

```bash
rg "search text"
rg -n "search text" src/
rg -n --hidden --glob '!*.git/*' "TODO" .
```

## Useful patterns

```bash
rg "class MyKernel" .
rg "import taichi" .
rg "execute_on" .
rg "TODO|FIXME" .
```

## When to use it

Use `rg` when you need to find:

- where a function is defined;
- where a config key is used;
- examples of a MOOSE input parameter;
- LaTeX labels and citations;
- TODO comments.

## Search hygiene

Search for specific words first. If nothing appears, broaden the query.
