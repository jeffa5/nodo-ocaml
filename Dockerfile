FROM ocaml/opam2:latest

COPY . .

RUN opam install .
