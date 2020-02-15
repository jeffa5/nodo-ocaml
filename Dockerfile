FROM ocaml/opam2:latest AS base
RUN opam update
COPY . .
RUN opam pin add --dev-repo --no-action bisect_ppx \
    && opam pin add . --no-action

FROM base
RUN opam depext --install nodo

FROM base
RUN opam depext --install nodo-filesystem

FROM base
RUN opam depext --install nodo-filesystem

FROM base
RUN opam depext --install nodo-cli-lib

FROM base
RUN opam depext --install nodo-cli
