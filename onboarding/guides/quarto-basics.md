# Quarto basics

Quarto uses `.qmd` files to create computational documents, reports, slides, and websites.

## When to use Quarto

Use Quarto for:

- computational reports;
- reproducible analysis notes;
- mixed text, code, figures, and equations;
- internal documentation that benefits from rendered output.

## Basic command

```bash
quarto render report.qmd
```

## Minimal `.qmd`

```markdown
---
title: "Report"
format: html
---

# Introduction

Some text.

```{python}
print("hello")
```
```

## Good habits

- Commit the `.qmd` source.
- Decide whether rendered outputs belong in Git.
- Keep data paths clear and relative when possible.
- Use environments documented by `uv`, conda, or containers.
