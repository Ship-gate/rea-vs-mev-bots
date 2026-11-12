FROM node:24-bookworm@sha256:14078204b6bb268c107ceff6d3c43f45a888105e5c2b18b14c32b3409d93a1b0

RUN command -v prlimit && command -v ps \
    && npm install --prefix /opt/rea --ignore-scripts --no-audit --no-fund rea-agents@6.7.0

USER node
ENTRYPOINT ["/opt/rea/node_modules/.bin/rea"]
