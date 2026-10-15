# Staging Release — Required Fixes

Findings from reviewing the current working-tree diff (service pruning: `booking`,
`order`, `payment`, `product`, `property`, `stores` removed) against the rest of
the gateway/service code. Ordered by severity. Check items off as they land.

---

## Critical

### 1. Secrets are committed to git history
`.env` files for every service (DB passwords, `INTERNAL_SECRET_KEY`) and the root
`.env` (`JWT_SECRET`) are tracked in git and are being edited in place with new
real values in the current diff.

Adding `.env` to `.gitignore` (already done in this diff) only stops **future**
commits — it does not untrack the files or remove them from history. The
secrets currently in the repo are already exposed to anyone with clone/history
access.

**Fix:**
- [x] Untracked every `.env` currently in git (`git rm --cached`) — working-tree
  files are kept, only the git index entry is removed. Note: `follow/.env` was
  never tracked to begin with, so it didn't need this step.
- [x] Added `.env.example` per surviving service (root, `authentication`,
  `comment`, `post`, `storage`, `user`, `vote`, `follow`) with placeholder
  values, so the shape of required config stays documented.
- [x] **Rotated `JWT_SECRET` and `INTERNAL_SECRET_KEY`** to new
  `openssl rand -hex 32` values across the root `.env`, `authentication/.env`,
  and every downstream service's `.env` — the old committed values are treated
  as burned. `DB_PASSWORD` values were left as-is since they only matter for
  local MongoDB; rotate them too if that Mongo instance is shared/reachable
  beyond your machine.
- [ ] Decide whether to scrub git history (`git filter-repo` / BFG). If the
  repo has ever been public or shared beyond the team, this is not optional —
  untracking going forward does not remove the old values already pushed to
  `origin/main`.

### 2. Shared secrets are hardcoded in gateway source and forwarded on every request
[`src/routing/gateway.rs:54`](src/routing/gateway.rs#L54) and
[`gateway.rs:59`](src/routing/gateway.rs#L59):

```rust
headers.insert("X-Service-Key", "7d363c8d...".parse().unwrap());
...
headers.insert("x-jwt-secret", "123456789".parse().unwrap());
```

Problems:
- The JWT **signing secret** is forwarded as a plaintext header to whatever
  backend the request routes to, on every single call — not just login. See
  [`authentication/handlers.rs:65`](src/services/authentication/src/handlers.rs#L65),
  which reads that header to sign new tokens.
- It's hardcoded, decoupled from the `JWT_SECRET` env var the gateway itself
  validates incoming tokens against ([`main.rs:40`](src/main.rs#L40)). Rotating
  `JWT_SECRET` silently breaks login without anyone touching `gateway.rs`.
- `X-Service-Key` is likewise a literal string in source rather than sourced
  from env, so rotating it means a code change + redeploy instead of a config
  change.

**Fix:**
- [x] `ServiceState` now carries `service_key: String`
  ([`routing/mod.rs`](src/routing/mod.rs)), populated from the
  `INTERNAL_SECRET_KEY` env var at gateway startup
  ([`main.rs`](src/main.rs#L20)) instead of a literal in `gateway.rs`.
- [x] Stopped forwarding the JWT signing secret as a per-request header
  entirely. The `authentication` service now reads `JWT_SECRET` from its own
  `.env` into `AppState` ([`authentication/main.rs`](src/services/authentication/src/main.rs))
  and `login` uses `state.jwt_secret` directly
  ([`authentication/handlers.rs`](src/services/authentication/src/handlers.rs))
  — no more `x-jwt-secret` header, in either direction.
- [x] `X-Service-Key` is now sourced from env (`INTERNAL_SECRET_KEY`) on both
  the gateway and every downstream service, and rotated (see #1).

Both `gateway` and `authentication` crates build clean after these changes
(`cargo build -p gateway`, `cargo build` in `src/services/authentication`).

---

## High

### 3. Dead routes to services deleted in this diff
`property` and `order` were deleted, but the gateway still knows about them:
- [`src/utils.rs:19-20`](src/utils.rs#L19-L20) — `detect_service` still maps
  `/properties` → `"property"`, `/orders` → `"order"`.
- [`src/routing/mod.rs`](src/routing/mod.rs) — `ServiceState` still carries
  `property_counter` / `order_counter`.
- [`src/routing/load_balancer.rs:8-9`](src/routing/load_balancer.rs#L8-L9) —
  `PROPERTY_BACKENDS` points at `localhost:8081`, which is `authentication`'s
  port. A stray request to `/api/v1/properties` gets silently forwarded into
  the auth service instead of a clean 404.

**Fix:**
- [x] Removed the `property`/`order` entries from `detect_service`
  ([`utils.rs`](src/utils.rs)), `ServiceState`
  ([`routing/mod.rs`](src/routing/mod.rs)), and `PROPERTY_BACKENDS`/
  `ORDER_BACKENDS` from ([`load_balancer.rs`](src/routing/load_balancer.rs)).
  Those paths now 404 cleanly through the normal "Service not found" path
  instead of being silently forwarded into `authentication`'s port.

### 4. Port collisions / mismatches between service `.env` and gateway routing
| Service | `.env` PORT | Gateway sends "X" traffic to | Match? |
|---|---|---|---|
| `follow` | 8083 | 8085 (`FOLLOW_BACKENDS`) | ❌ mismatch |
| `comment` | 8083 | 8083 (`COMMENT_BACKENDS`) | ⚠️ collides with `follow` |
| `post` | 8086 | 8082 (`POST_BACKENDS`) | ❌ mismatch |
| `storage` | 8086 | — (not routed) | ⚠️ collides with `post` |

`follow` and `comment` can't both bind 8083 at once, and even if they could,
the gateway is sending `follow` traffic to a port `follow` doesn't listen on.
Same problem for `post`/`storage`.

**Fix:**
- [x] Reassigned ports to match each binary's own hardcoded fallback default
  (so `.env` stops fighting the code) and `load_balancer.rs`:
  `user` 8080, `authentication` 8081, `post` 8082 (was 8086),
  `comment` 8083, `vote` 8084, `follow` 8085 (was 8083), `storage` 8086
  (unchanged — not routed through the gateway, no longer collides with
  `post`). Verified every `.env` PORT now matches its `load_balancer.rs`
  entry exactly (see table below).
- [ ] No automated check ties `.env` PORT values to the load balancer table
  yet, so they can still drift apart silently in the future — worth adding a
  startup assertion or a single source of truth (e.g. generate both from one
  config) if this keeps happening.

| Service | `.env` PORT | Gateway backend |
|---|---|---|
| `user` | 8080 | 8080 |
| `authentication` | 8081 | 8081 |
| `post` | 8082 | 8082 |
| `comment` | 8083 | 8083 |
| `vote` | 8084 | 8084 |
| `follow` | 8085 | 8085 |
| `storage` | 8086 | — (not gateway-routed) |

---

## Medium

### 5. Dockerfile bakes `.env` into the image
```dockerfile
# Copy .env if needed
COPY .env /app/.env
```
This puts secrets into an image layer, which is worse than the
`docker-compose.yml` `env_file:` approach already used for the same service —
anyone who can pull the image can extract the layer and read the secrets.

**Fix:**
- [x] Dropped the `COPY .env /app/.env` line from `Dockerfile`. Config is
  injected at runtime via `docker-compose.yml`'s existing `env_file: - .env`
  entry instead, so nothing changes operationally — the image just no longer
  bakes secrets into a layer.

### 6. TLS certs removed — confirm intentional
`certs/cert.pem` and `certs/key.pem` were deleted. Functionally harmless today
since `TlsMiddleware` ([`src/middleware/tls.rs`](src/middleware/tls.rs)) is an
unused no-op stub not wired into `main.rs`. Flagging in case staging expects
the gateway to terminate TLS itself rather than sitting behind a reverse proxy.

**Fix:**
- [x] Confirmed with the team: staging terminates TLS at a reverse
  proxy/load balancer in front of the gateway, which then talks plain HTTP to
  the gateway on `:8000`. No code change needed — `TlsMiddleware` stays an
  unused stub (or can be deleted outright as follow-up cleanup) and the
  removed `certs/` files are correctly not needed.

---

## CI/CD

Added two GitHub Actions workflows:

- [`.github/workflows/ci.yml`](.github/workflows/ci.yml) — on every push and
  PR, builds and `cargo test`s all 8 crates (gateway + 7 services) in a
  matrix. `cargo fmt --check` and `cargo clippy` run too but are
  `continue-on-error: true` for now since the codebase isn't clippy/fmt-clean
  yet — flip that off once it is, so lint failures actually block merges.
- [`.github/workflows/deploy-staging.yml`](.github/workflows/deploy-staging.yml)
  — on push to `main` (or manual `workflow_dispatch`): builds a release binary
  per crate, assembles them into the same relative layout `server.sh` expects
  (`target/release/gateway`, `src/services/<name>/target/release/<name>`),
  rsyncs that plus `server.sh` itself to the staging host over SSH, runs
  `./server.sh restart` remotely, then curls `/health` on the host as a smoke
  test. It never rsyncs `.env` files or uses `--delete`, so it can't touch
  secrets or anything else already on the box.

**Deployment strategy note:** this matches the repo's existing bare-VM /
`server.sh` process model rather than introducing Docker Compose for all
services (only `gateway` has a `Dockerfile` today). Revisit if staging moves
to containers later.

**`server.sh` had two bugs that would have broken every deploy — fixed:**
- `declare -A` (associative arrays) requires bash 4+; macOS's stock
  `/bin/bash` is 3.2 and silently mis-executes it (every `SERVICES["name"]=`
  assignment collided into a single numeric index instead of erroring), so
  the whole service list collapsed to just the last entry. Rewrote it to use
  parallel indexed arrays (`SERVICE_NAMES`, `SERVICE_PATHS`, ...), which work
  on both bash 3.2 and modern bash — confirmed by actually running
  `./server.sh build`/`start`/`stop` locally.
- `start_services` computed each service's `.env` location from its *binary's*
  directory (`src/services/user/target/release/.env`, which doesn't exist)
  instead of the crate root (`src/services/user/.env`). Every service except
  `gateway` panicked on startup (`env::var("DB_USERNAME").unwrap()` →
  `NotPresent`) — `gateway` only "worked" because its own `dotenv()` call
  happens to find the repo-root `.env` via the process's CWD regardless of
  what `server.sh` sourced. Fixed with an explicit `SERVICE_ENV_DIRS` array
  pointing at each crate root. Re-verified end-to-end: all 7 services + the
  gateway start, bind their correct ports, and a request through
  `http://localhost:8000/api/v1/auth/register` correctly routes to
  `authentication`.
- Added `build` and `dev` subcommands (`./server.sh dev` = build every crate
  in release mode, then start all of them — the "one command" ask).

**One-time staging host setup (not automated — do this once, manually):**
- [ ] Create a deploy user on the staging host and generate an SSH keypair for
  it (`ssh-keygen -t ed25519 -f deploy_key -N ""`); add the public key to that
  user's `~/.ssh/authorized_keys`.
- [ ] Pick a deploy directory (e.g. `/opt/microservice-with-rust`) and create
  it: `mkdir -p /opt/microservice-with-rust`.
- [ ] Place each service's real `.env` at
  `<deploy dir>/src/services/<name>/.env` and the gateway's at
  `<deploy dir>/.env`, using the rotated secrets from #1/#2 above (see the
  `.env.example` files added alongside each service for the expected shape).
  CI never touches these.
- [ ] Confirm `rsync` is installed on the staging host (`apt-get install
  -y rsync` if not).

**GitHub repo configuration:**
- [ ] Create a `staging` [Environment](../../settings/environments) (used by
  `deploy-staging.yml`'s `environment: staging` — lets you require manual
  approval on deploys if you want that gate later).
- [ ] Add these as repo or environment secrets:
  | Secret | Value |
  |---|---|
  | `STAGING_HOST` | staging host IP/hostname |
  | `STAGING_USER` | the deploy user created above |
  | `STAGING_SSH_KEY` | the deploy user's **private** key (full contents of `deploy_key`) |
  | `STAGING_SSH_PORT` | optional, defaults to `22` if unset |
  | `STAGING_DEPLOY_PATH` | the deploy directory chosen above, e.g. `/opt/microservice-with-rust` |

**Assumption:** the staging host is `x86_64` Linux (matches GitHub-hosted
`ubuntu-latest` runners, which build the release binaries with no
cross-compilation). If staging is ARM, the build steps need a cross target
(e.g. `aarch64-unknown-linux-gnu`) added before this will work.

---

## Minor

### 7. `comment` service ignores its own `DB_NAME` env var
[`src/services/comment/src/db.rs`](src/services/comment/src/db.rs) hardcodes
`client.database("social_media_db")` instead of reading `DB_NAME`, unlike
`authentication`, `user`, `vote`, and `post`, which all do
`env::var("DB_NAME")`. Currently harmless because the values happen to match,
but it'll silently ignore future config changes.

**Fix:**
- [x] `comment/src/db.rs` now uses `client.database(&db_name)` (the value it
  was already reading from `DB_NAME` at the top of the function, previously
  unused) instead of the hardcoded literal, matching every sibling service.
  Builds clean; behavior is unchanged today since the `.env` value already
  matched.

---

## Suggested order of operations
1. Rotate all secrets, untrack `.env` files (#1), remove hardcoded secrets
   from `gateway.rs` (#2) — do these together since #2's fix requires new env
   vars that replace the values being rotated in #1.
2. Clean up dead `property`/`order` routing (#3).
3. Fix port assignments (#4) and verify with a real `server.sh start` that
   every service comes up and the gateway can reach it end-to-end.
4. Dockerfile / TLS cleanup (#5, #6).
5. `comment` DB_NAME fix (#7).
