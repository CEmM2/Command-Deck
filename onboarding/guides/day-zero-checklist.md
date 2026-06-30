# Day 0 checklist

This is the minimum setup before useful work starts.

## Accounts

- GitHub account with access to the lab repositories.
- Google or institutional account for GCP access, if relevant.
- Overleaf access for shared papers.
- Zotero account for shared libraries.
- Teams or the lab communication platform.

## Local machine

Install or confirm:

```bash
git --version
gh --version
uv --version
python --version
rg --version
```

Optional depending on project:

```bash
mamba --version
conda --version
docker --version
quarto --version
gcloud --version
```

## GitHub auth

Use browser-based GitHub CLI auth first:

```bash
gh auth login --web --git-protocol ssh
```

Avoid pasting personal access tokens into shell commands, notebooks, `.env` files, shared scripts, or screenshots.

## First successful workflow

1. Clone a lab repository.
2. Create a branch.
3. Make a tiny documentation edit.
4. Commit it.
5. Push it.
6. Open a pull request.
