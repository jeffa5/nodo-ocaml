FROM ocaml/opam2:latest

RUN opam update

COPY . .

RUN opam install .
