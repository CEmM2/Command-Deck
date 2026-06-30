# `git` vs `gh`

`git` and `gh` are different tools.

## `git`

`git` works with the repository itself:

```bash
git status
git add file.py
git commit -m "message"
git push
git pull
git branch
git switch
```

Use `git` for version control: commits, branches, merges, history, and remotes.

## `gh`

`gh` is the GitHub CLI. It talks to GitHub:

```bash
gh auth status
gh repo clone owner/repo
gh pr create --web
gh pr list
gh issue list
```

Use `gh` for GitHub-specific actions: authentication, pull requests, issues, repository browsing, and web handoffs.

## Rule of thumb

- If the action changes local history or files, it is probably `git`.
- If the action talks to GitHub as a platform, it is probably `gh`.

Some workflows use both. For example, `git push` sends a branch to GitHub, then `gh pr create` opens a pull request from that branch.
