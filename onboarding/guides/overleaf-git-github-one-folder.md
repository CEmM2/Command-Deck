# Overleaf and GitHub from one folder

A paper can have one local folder with two Git remotes:

- `origin`: GitHub
- `overleaf`: Overleaf Git remote

## Setup

Clone or create the GitHub repository first, then add Overleaf:

```bash
git remote add origin git@github.com:OWNER/REPO.git
git remote add overleaf https://git.overleaf.com/PROJECT_ID
git remote -v
```

If `origin` already exists, do not add it again. Check first:

```bash
git remote -v
```

## Pull from Overleaf

```bash
git pull overleaf master
```

Some Overleaf projects may use another branch name. Check the Overleaf Git instructions.

## Push to GitHub

```bash
git push origin main
```

## Practical workflow

1. Pull from Overleaf.
2. Resolve conflicts locally if needed.
3. Commit clean changes.
4. Push to GitHub.
5. Push to Overleaf only when you intend to update the Overleaf project.

## Warnings

- Avoid simultaneous large edits in Overleaf and local Git.
- Keep generated files out unless needed.
- Make sure figures are committed or uploaded consistently.
