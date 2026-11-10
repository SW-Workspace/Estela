$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
foreach ($arguments in @(@("fmt", "--all", "--", "--check"), @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings"), @("test", "--workspace"), @("xtask", "check-deps"), @("deny", "check"), @("xtask", "check-sqlx"))) {
  & cargo @arguments
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
