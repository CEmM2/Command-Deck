# Security Policy

Command Deck is a local command launcher. It is not a sandbox.

Templates can run arbitrary shell commands as the current user. Only use templates from sources you trust, and review generated commands before executing them. Treat `.toml` template files as executable code.
