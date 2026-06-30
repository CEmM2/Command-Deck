# Safe GitHub authentication

The goal is to authenticate without exposing tokens in shell history, notebooks, environment files, screenshots, or shared scripts.

## Recommended first path

Use GitHub CLI's browser flow:

```bash
gh auth login --web --git-protocol ssh
```

Then check:

```bash
gh auth status
ssh -T git@github.com
```

## What not to do

Do not paste personal access tokens into:

- `.bashrc`, `.zshrc`, or `.profile`;
- `.env` files committed to Git;
- notebooks;
- scripts shared with the lab;
- screenshots;
- terminal commands that may be saved in shell history.

## SSH keys

Prefer SSH keys for Git operations. Protect private keys. Never commit files like:

```text
id_rsa
id_ed25519
*.pem
```

## Personal access tokens

Use PATs only when needed. Prefer short-lived, fine-scoped tokens. Store them in a credential manager, not in the project directory.

## If a token leaks

1. Revoke it immediately on GitHub.
2. Remove it from the repository history if committed.
3. Tell the supervisor or repo owner.
4. Rotate any related credentials.
