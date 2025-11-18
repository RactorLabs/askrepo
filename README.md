# AskRepo Service

`askrepo-service` is a small Rust application that polls Twitter mentions for a target account and provisions TSBX sandboxes to answer repository questions referenced in those tweets.

## How It Works

- Every 60 seconds (configurable), the service calls `GET /2/users/:id/mentions` on the Twitter API using the provided user identifier.
- Each mention is mapped to a sandbox tagged `tweet-<tweet_id>`. Existing sandboxes are reused; new tweets trigger a fresh sandbox creation through the TSBX API.
- Before acting on a mention, AskRepo checks the conversation for prior replies from the monitored account and skips tweets that already have an AskRepo response.
- The newly created sandbox receives:
  - Metadata describing the source tweet.
- A guardrail-focused instruction set directing the sandbox to clone the `ractorlabs/twitter_api_client` repository, gather the conversation thread, vet the request, inspect the referenced repository, and reply using the Twitter client tooling.
  - The `tweet_id` as the initial prompt.

## Required Environment Variables

| Variable | Description |
| --- | --- |
| `TSBX_HOST_URL` | Base URL for the TSBX API (e.g., `http://localhost:9000`). |
| `TSBX_ADMIN_TOKEN` | Operator token with permission to create sandboxes and tasks. |
| `TWITTER_BEARER_TOKEN` | Twitter API v2 bearer token. |
| `TWITTER_USER_ID` | Twitter numeric user id whose mentions should be polled. |
| `TWITTER_API_KEY` | OAuth consumer key shared with sandboxes to post replies. |
| `TWITTER_API_SECRET` | OAuth consumer secret associated with the key above. |
| `TWITTER_ACCESS_TOKEN` | OAuth access token that authorizes tweet posting. |
| `TWITTER_ACCESS_TOKEN_SECRET` | OAuth access token secret for signing requests. |

### Optional Environment Variables

| Variable | Default | Description |
| --- | --- | --- |
| `TWITTER_API_BASE` | `https://api.x.com` | Override for the Twitter API base URL. |
| `TWITTER_POLL_INTERVAL_SECS` | `90` | Poll cadence in seconds (minimum 10s enforced). |
| `TWITTER_SINCE_ID` | unset | Seed `since_id` to skip older mentions on startup. |

## Running Locally
>
> Requires Rust 1.82 or newer (`rustup update stable`).

```bash
cargo run
```

The service listens for `Ctrl+C` and will exit gracefully.

## Container Usage

- Build the image: `./scripts/build.sh` (pass `--no-cache` or `--tag` as needed).
- Start the container: `./scripts/start.sh` (reads configuration from `.env` in the project root).
- To run detached: `./scripts/start.sh --detach`.
- Inspect logs for a detached run: `docker logs askrepo -f`.

## Notes

- Tags applied to provisioned sandboxes: `askrepo`, `twitter`, and `tweet<tweet_id>`.
- Sandboxes are created with a startup task containing the `tweet_id`, which becomes the first TSBX task for that workspace.
- Twitter API rate limits are surfaced via logs; the service will retry on the next polling interval.
- When present, the Twitter credentials listed above are copied into the sandbox `.env` file as `TWITTER_*` keys so the `twitter_api_client` tooling can authenticate.
