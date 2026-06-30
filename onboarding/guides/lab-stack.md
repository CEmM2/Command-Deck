# Common lab stack

These are tools you will see repeatedly in the lab. Some are for code. Many are not.

## Core tools

- **Git**: version control for text-based projects.
- **GitHub**: hosting, collaboration, pull requests, issues, and code review.
- **GitHub CLI (`gh`)**: command-line interface for GitHub operations.
- **Terminal**: the basic interface for local and remote scientific computing.
- **uv**: fast Python project and dependency manager.
- **conda/mamba/minimamba**: environments that can include Python and non-Python compiled packages.
- **Spack**: HPC-oriented package/build manager for compiled scientific software.
- **Docker**: local container engine.
- **Singularity/Apptainer**: container workflow commonly accepted on HPC systems.
- **GCP**: cloud VMs and storage when local or cluster resources are not enough.

## Research and writing tools

- **Zotero**: reference manager and shared bibliography workflow.
- **Overleaf**: collaborative LaTeX writing.
- **Quarto**: computational documents and reports.
- **Teams**: communication, meetings, files, and institutional entropy.

## Rule of thumb

When a tool stores code or text, prefer keeping the source in Git. When a tool stores large binary data, keep the data location documented and avoid committing the data itself unless the repository is explicitly designed for that.
