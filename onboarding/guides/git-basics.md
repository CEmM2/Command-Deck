# Git basics

Git is version control. It records snapshots of a project over time so you can understand what changed, who changed it, and why.

## Why we use it

We use Git because research projects involve code, scripts, papers, configs, input files, and documentation that change constantly. Without version control, collaboration becomes folders named `final`, `final2`, and `really_final_this_time`.

## Mental model

- **Repository**: a project tracked by Git.
- **Commit**: a saved snapshot with a message.
- **Branch**: a movable line of work.
- **Remote**: a copy of the repository elsewhere, often GitHub.
- **Pull request**: a GitHub discussion around merging a branch.

## Daily workflow

```bash
git status --short --branch
git switch -c feature/my-change
# edit files
git add path/to/file
git commit -m "Explain the change"
git push -u origin HEAD
gh pr create --web
```

## Safe habits

- Run `git status` often.
- Pull with `git pull --ff-only` when you expect only a simple update.
- Make small commits with meaningful messages.
- Do not commit secrets, tokens, large results, or temporary output.
- Ask before force-pushing shared branches.

## Common confusion

Git is local. GitHub is a hosting/collaboration service. You can use Git without GitHub, but in the lab they usually appear together.
