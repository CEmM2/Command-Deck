# conda, mamba, and minimamba basics

Conda-style environments can include Python packages and non-Python compiled dependencies. This is why they remain useful in scientific computing, despite the emotional cost.

## Names

- **conda**: package and environment manager.
- **mamba**: faster conda-compatible solver and installer.
- **miniconda/miniforge/minimamba**: smaller installers for conda-style environments.

## Basic commands

```bash
mamba create -n lab-env python=3.11
mamba activate lab-env
mamba install numpy scipy
mamba env list
mamba deactivate
```

If `mamba` is unavailable, use `conda` for the same basic structure.

## Environment files

A typical `environment.yml`:

```yaml
name: lab-env
channels:
  - conda-forge
dependencies:
  - python=3.11
  - numpy
  - scipy
  - pip
```

Create from it:

```bash
mamba env create -f environment.yml
```

## Good habits

- Name environments clearly.
- Export only intentional dependencies.
- Avoid installing into `base`.
- Do not mix `pip` and `mamba` randomly. If you must use `pip`, do it after the conda packages are installed.
