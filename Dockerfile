FROM ocurrent/opam:alpine-3.10-ocaml-4.09 AS base

COPY . .
RUN opam pin add . --no-action

FROM base
RUN opam depext --install nodo

FROM base
RUN opam pin add --dev-repo --no-action omd
RUN opam depext --install nodo-cli
