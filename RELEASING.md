# Releasing forgr

This is the procedure an agent follows to release forgr. It has no operator
step after the first release: no confirmation, no choice. Run the steps in
order. On the first failure, stop and report it as the Failure section says.

Config is `release.toml`. The mechanics are `scripts/release.sh` and
`scripts/sync-version.sh`, copied from rafters-studio/migratr (commit 20b5f45),
which copied them from runlegion/legion (legion 0.43.5, commit 58504b1c) with
two patches: the version bump also moves the `version` pin on every
`path = "..."` dependency, and `--activate` is refused. Check their behavior
with `bash scripts/test-release.sh`.

forgr is a library: a release publishes forgr-schema, forgr-macros and forgr to
crates.io, and builds no binaries.

Replace `X.Y.Z` with the new version, `<level>` with `patch`, `minor` or
`major`, and `<pr>` with the release PR number.

## First release: the operator publishes

crates.io trusted publishing can only be configured for a crate that already
exists, so the first publish of each crate is done by the operator with
`cargo publish`, not by `release.yml`. The first release is 0.1.0: the version
in `Cargo.toml` and the `## 0.1.0` entry in `CHANGELOG.md` are already on
`main`, so `release.sh` is not run for it (it refuses a non-increment).

On `main`, at the commit that carries the `## 0.1.0` entry, logged in with
`cargo login`:

```bash
cargo publish -p forgr-schema
cargo publish -p forgr-macros
cargo publish -p forgr
```

In that order: forgr-macros needs forgr-schema on crates.io, and forgr needs
both. Then, for each of the three crates on crates.io, add a trusted publisher:
repository `rafters-studio/forgr`, workflow `release.yml`.

Tag that commit and push the tag:

```bash
git tag -a v0.1.0 -m "forgr 0.1.0" <sha>
git push origin v0.1.0
```

`release.yml` runs on the tag, finds each crate's 0.1.0 already on crates.io,
and skips it. Every later release follows the steps below.

## Keeping state

A release can outlive one agent session. After every step that creates something
(the release issue, the release PR, the tag), comment it on the release issue.
To resume, read that issue's comments, check what exists, and continue at the
first step not done.

## 1. Preconditions

On `main`, in sync with origin, no uncommitted changes to tracked files.
Untracked files are ignored, as `release.sh`'s own guard ignores them.

```bash
git rev-parse --abbrev-ref HEAD                      # main
git fetch origin main && git status -sb              # no "ahead"/"behind"
git status --porcelain --untracked-files=no          # empty, or only CHANGELOG.md when resuming after step 2
git branch --list 'release/*'                        # empty
```

If any fails, stop and report which. Change nothing.

## 2. Changelog

Dispatch the `legion:changelog` agent for forgr. It prepends `## X.Y.Z` to
`CHANGELOG.md` with a line `<Patch|Minor|Major> release: <rationale>`. The bump
level is that word, lowercased.

Tell it to put the `<Patch|Minor|Major> release: <rationale>` line as the first
paragraph under the heading. Read the result back from `CHANGELOG.md`: `X.Y.Z` is
the first `## X.Y.Z` heading, and `<level>` is the word before `release:` in the
first paragraph under it, lowercased. If the agent failed, or the entry has no
such heading and line, stop and report it.

Leave the entry uncommitted. `release.sh` takes it into the release commit.

## 3. Release issue

```bash
gh issue create --repo rafters-studio/forgr \
  --title "chore(release): vX.Y.Z" \
  --body "$(printf '## Acceptance criteria\n\n- forgr X.Y.Z is tagged, and forgr-schema, forgr-macros and forgr X.Y.Z are on crates.io\n')"
```

The pr-write gate needs an issue to map. Note the issue number.

## 4. Stage the release commit

```bash
scripts/release.sh <level>
```

It runs the preflight commands from `release.toml`, bumps `Cargo.toml`
(`[workspace.package]` and the two path-dependency pins), refreshes
`Cargo.lock`, commits `chore(release): X.Y.Z` on `release/X.Y.Z`, and pushes that
branch. It refuses when the `## X.Y.Z` heading does not equal the version it
computes from `<level>`; stop and report its message.

## 5. Gates, PR, merge queue

On `release/X.Y.Z`, run `/legion:legion-simplify`, then `/legion:legion-pr-write`
(both are keyed to the release commit). pr-write runs
`legion pr write-check --repo forgr --issue <issue>` on a body file, BODY, kept
in a scratch directory outside the repo, that maps the release issue's criterion.
Then open the PR with that body; its output includes `created PR #<pr>`, which
gives `<pr>`:

```bash
legion pr create --repo forgr --title "chore(release): vX.Y.Z" --head release/X.Y.Z --closes <issue> --body "$(cat BODY)"
legion pr merge --repo forgr --number <pr>
```

## 6. Tag

```bash
scripts/release.sh X.Y.Z --finish=<pr>
```

It waits for the merge, tags the merged release commit, and pushes the tag. The
tag runs `release.yml`, which publishes the three crates to crates.io.

## 7. Wait for the publish

Poll up to 90 times, 20 seconds apart, until the `release.yml` run for `vX.Y.Z`
has completed, and require it to have succeeded. This uses `gh`, not `curl`:
legion's hook stops `curl` for operator approval, which would stall the release.

```bash
for i in $(seq 90); do
  run="$(gh run list --repo rafters-studio/forgr --workflow release.yml --branch vX.Y.Z --limit 1 \
    --json status,conclusion,url --jq '.[0] | "\(.status) \(.conclusion) \(.url)"' 2>/dev/null)"
  case "$run" in completed\ *) break ;; esac
  sleep 20
done
case "$run" in
  "completed success "*) ;;
  *) echo "release.yml has not published vX.Y.Z: $run"; exit 1 ;;
esac
```

On timeout or failure, report "release.yml has not published vX.Y.Z" with the
Actions run URL.

## 8. Report

Report: the version, the release PR, the tagged sha and the `release.yml` run.

## Failure

- Step 1 fails: stop and report which precondition. Nothing has changed.
- The `## X.Y.Z` heading disagrees with the version `release.sh` computes from
  `<level>`: it refuses at its header check. Stop and report both versions.
- Release-side failures (queue ejection, merge timeout, tag failure): report
  `release.sh`'s message verbatim and stop.
- A release-side gate fails (simplify or pr-write in step 5): stop and report
  the gate's output, the release issue and the pushed `release/X.Y.Z` branch.
  `release.sh` refuses to stage again while that branch exists, so a later run
  first deletes the unmerged branch and reuses or closes the issue.
- Step 7 times out or the run fails: report "release.yml has not published
  vX.Y.Z" with the Actions run URL. `release.yml` skips any crate whose version
  is already on crates.io, so re-running the run publishes only what is
  missing.
