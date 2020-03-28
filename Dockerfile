FROM ocaml/opam2:latest AS base

COPY . .
RUN opam pin add . --no-action

FROM base
RUN opam depext --install nodo

FROM base
RUN opam pin add --dev-repo --no-action omd
RUN opam depext --install nodo-markdown

FROM base
RUN opam depext --install nodo-json

FROM base
RUN opam depext --install nodo-filesystem

FROM base
RUN opam depext --install nodo-git-filesystem

FROM base
RUN opam depext --install nodo-cli-lib

FROM base
RUN opam pin add --dev-repo --no-action omd
RUN opam depext --install nodo-cli
