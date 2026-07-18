# Showcase PDF

Regenerate the showcase artifacts, then compile the document from this directory:

```sh
cargo run -p vz-adapter-python --example showcase
latexmk -pdf -interaction=nonstopmode -halt-on-error showcase.tex
cp showcase.pdf ../showcase.pdf
```

The document uses TikZ for vector diagrams and `listings` for captured source
and JSON artifacts. It reads directly from `../gallery`.
