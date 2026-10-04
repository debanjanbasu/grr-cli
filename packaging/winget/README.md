# winget packaging

The manifests under `winget-pkgs/` are generated per release — never edited
by hand:

```sh
node scripts/generate-winget.ts v0.8.1        # writes winget-pkgs/*.yaml
```

The generator downloads the release's Windows zips, verifies them against the
release's own `SHA256SUMS`, reads the nested executable's name out of the
archive, and derives every manifest field from that. Generate, don't retype:
the first hand-written submission promised `grr.exe` while the archive
contained `windows-x86_64.exe`, so winget's validation could never find the
nested installer and looped on "issue with installing the application
correctly" (winget-pkgs PR #441361).

The same three files ride every release automatically as the
`grr-<tag>-winget-manifests.tar.gz` asset (release-binaries.yml, `winget`
job). Submitting is a copy into a `winget-pkgs` fork at
`manifests/d/debanjanbasu/grr/<version>/` and opening a PR.
