FROM ocaml/opam2:latest AS base
RUN opam update
COPY . .

FROM base
RUN opam install nodo

FROM base
RUN opam install nodo-markdown

FROM base
RUN opam install nodo-filesystem

FROM base
RUN opam install nodo-cli-lib

FROM base
RUN opam install nodo-cli
