# LaTeX basics

LaTeX is a markup system for writing technical documents. It is common in papers with equations, references, figures, and strict journal formats.

## Important files

- `main.tex`: main source file.
- `.bib`: bibliography database.
- `.sty`: style file.
- `.cls`: document class.
- `.pdf`: compiled output.
- Auxiliary files: generated during compilation and usually not important to edit.

## Basic structure

```tex
\documentclass{article}
\begin{document}
Hello.
\end{document}
```

## Compile locally

```bash
latexmk -pdf main.tex
latexmk -c main.tex
```

## Good habits

- Keep one clear main `.tex` file.
- Use labels for figures, tables, and equations.
- Store references in `.bib`.
- Do not manually edit generated auxiliary files.
- Keep figures in a predictable folder.
