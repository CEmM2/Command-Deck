# File types

File extensions are conventions. They do not guarantee correctness, but they help tools and humans guess what a file is for.

## Writing and docs

- `.tex`: LaTeX source for papers and reports.
- `.bib`: bibliography database used by LaTeX and reference managers.
- `.md`: Markdown notes, README files, simple documentation.
- `.qmd`: Quarto Markdown, often used for computational reports.
- `.css`: styling rules for HTML or Quarto output.

## Config and structured text

- `.json`: strict structured data. Good for machine output and APIs.
- `.yaml` / `.yml`: human-readable config, common in CI, environments, and workflows.
- `.toml`: readable config with clear types; used by Python tooling, Rust, and Command Deck templates.

## Data and scientific output

- `.csv`: plain-text table. Easy to inspect, bad for huge or complex data.
- `.h5` / `.hdf5`: hierarchical binary scientific data.
- `.xdmf`, `.vtk`, `.vtu`: visualization and mesh/output formats used by scientific tools.

## Note about `fst5`

If someone wrote `fst5`, confirm whether they meant `hdf5`, `f5`, `fast5`, or a lab-specific format. Do not propagate mystery file types through documentation unless the goal is folklore.
