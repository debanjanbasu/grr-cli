# grr in a container — for MCP introspection hosts (Glama, Docker-based
# clients) and anyone who wants `grr mcp` without installing the binary.
#
# The latest release is resolved at build time rather than pinned, so the
# image never rots behind a release; the release assets are public and the
# linux archives carry the binary as `grr` (AGENTS invariant on archive
# members). `grr mcp` speaks stdio MCP and needs no credentials for
# initialize / tools-list; authenticated calls still need a login volume or
# an embedded token.
FROM debian:stable-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl jq zstd \
 && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
RUN set -eux; \
    case "${TARGETARCH:-amd64}" in \
      amd64) ARCH=x86_64 ;; \
      arm64) ARCH=aarch64 ;; \
      *) echo "unsupported TARGETARCH: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    TAG="$(curl -fsSL https://api.github.com/repos/debanjanbasu/grr-cli/releases/latest | jq -r .tag_name)"; \
    curl -fsSL -o /tmp/grr.tar.zst \
      "https://github.com/debanjanbasu/grr-cli/releases/download/${TAG}/grr-${TAG}-linux-${ARCH}.tar.zst"; \
    zstd -d /tmp/grr.tar.zst -o /tmp/grr.tar; \
    tar -xf /tmp/grr.tar -C /usr/local/bin grr; \
    chmod +x /usr/local/bin/grr; \
    rm -f /tmp/grr.tar /tmp/grr.tar.zst; \
    grr --version | head -1

ENTRYPOINT ["grr", "mcp"]
