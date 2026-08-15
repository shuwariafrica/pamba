#requires -Version 7.0

<#
.SYNOPSIS
  Creates and pushes a signed git release tag for the specified version.

.DESCRIPTION
  Validates the version against semantic versioning rules, ensures the working
  directory is clean and in sync with the main repository upstream, then creates
  a GPG-signed annotated tag and pushes it.

  Allowed pre-release classifiers (case-sensitive):
    - milestone.N (or m.N)
    - alpha.N
    - beta.N
    - rc.N
    - SNAPSHOT

.PARAMETER Version
  The semantic version to tag (e.g. 1.2.3, 1.2.3-alpha.1, 1.2.3-SNAPSHOT).

.EXAMPLE
  .\release.ps1 1.2.3
.EXAMPLE
  .\release.ps1 1.2.3-rc.1
#>

[CmdletBinding()]
param(
  [Parameter(Mandatory, Position = 0, HelpMessage = 'Semantic version (e.g. 1.2.3, 1.2.3-rc.1)')]
  [ValidateNotNullOrEmpty()]
  [string]$Version
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Display thrown errors cleanly and exit with code 1.
trap {
  $msg = if ($_.Exception -and $_.Exception.Message) { $_.Exception.Message } else { ($_ | Out-String).TrimEnd() }
  Write-Host "Error: $msg" -ForegroundColor Red
  exit 1
}

# --- Configuration -----------------------------------------------------------

$MainRepoOwner = if ($env:MAIN_REPO_OWNER) { $env:MAIN_REPO_OWNER } else { 'shuwariafrica' }
$MainRepoName = if ($env:MAIN_REPO_NAME) { $env:MAIN_REPO_NAME }  else { 'pamba' }
$MainRepoSlug = "$MainRepoOwner/$MainRepoName"

# --- Helper Functions --------------------------------------------------------

function Invoke-Git {
  <#
  .SYNOPSIS
    Runs a git command, returns stdout lines (as strings), and throws on non-zero exit.
  #>

  # Prevent the caller's $ErrorActionPreference = 'Stop' from triggering on
  # stderr ErrorRecords before we can inspect $LASTEXITCODE ourselves.
  $local:ErrorActionPreference = 'Continue'

  # Merge stderr into stdout, then normalise every object to a plain string.
  # This avoids relying on ErrorRecord type-checks which are host-dependent.
  $output = (& git @args 2>&1 | ForEach-Object { "$_" })
  $exitCode = $LASTEXITCODE

  if ($exitCode -ne 0) {
    $msg = "git $($args -join ' ') failed (exit $exitCode)"
    if ($output) { $msg += ": $($output -join "`n")" }
    throw $msg
  }

  return $output
}

function Get-GitHubRepoSlug([string]$Url) {
  <#
  .SYNOPSIS
    Extracts 'owner/repo' from common GitHub remote URL formats.
  .DESCRIPTION
    Handles HTTPS (with or without .git, trailing slash, embedded credentials),
    SSH shorthand (git@github.com:owner/repo), and SSH scheme (ssh://...).
    Returns $null if the URL does not match a GitHub remote.
  #>
  if (-not $Url) { return $null }

  $u = $Url.Trim()
  $u = $u -replace '/+$', ''                       # trailing slashes
  $u = $u -replace '\.git$', ''                    # .git suffix
  $u = $u -replace '^(https?://)[^@/]+@', '$1'     # embedded credentials

  if ($u -match 'github\.com[:/](?<slug>[^/]+/[^/]+)') {
    return $Matches['slug']
  }
  return $null
}

function Get-ScalarLine([object]$Lines) {
  <#
  .SYNOPSIS
    Coerces Invoke-Git output (null, scalar string, or array) to a single trimmed string.
  #>
  if ($null -eq $Lines) { return '' }
  $s = if ($Lines -is [string]) { $Lines } else { $Lines | Select-Object -First 1 }
  if ($s) { return "$s".Trim() } else { return '' }
}

# --- Prerequisites -----------------------------------------------------------

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
  throw 'git is not installed or not on PATH.'
}

Invoke-Git rev-parse --is-inside-work-tree | Out-Null

# --- Version validation ------------------------------------------------------

$SemverPattern = '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-((milestone|m|alpha|beta|rc)\.([1-9]\d*)|SNAPSHOT))?(\+[0-9A-Za-z\-]+(\.[0-9A-Za-z\-]+)*)?$'

if ($Version -cnotmatch $SemverPattern) {
  throw "Version '$Version' is not valid. Run with -? for allowed formats."
}

# --- Working tree checks -----------------------------------------------------

$status = Invoke-Git status --porcelain
if ($status) {
  git status
  throw 'Uncommitted changes present. Commit or stash before creating a release tag.'
}

Write-Host 'Working directory is clean.' -ForegroundColor Green

# --- Compute tag name --------------------------------------------------------

$TagName = "v$Version"

# --- Locate the main repository remote --------------------------------------

$MainRemote = $null
foreach ($r in (Invoke-Git remote)) {
  foreach ($u in (Invoke-Git remote get-url --all $r)) {
    if ((Get-GitHubRepoSlug $u) -ieq $MainRepoSlug) {
      $MainRemote = $r
      break
    }
  }
  if ($MainRemote) { break }
}

if (-not $MainRemote) {
  throw "No git remote points to $MainRepoSlug. Add one and try again."
}

# --- Branch & upstream checks ------------------------------------------------

$currentBranch = Get-ScalarLine (Invoke-Git rev-parse --abbrev-ref HEAD)
if ($currentBranch -eq 'HEAD') {
  throw 'Detached HEAD state. Check out a branch with an upstream tracking branch.'
}

# Query both remote and merge ref directly - a branch can have remote set
# without merge (rare), which would cause @{u} to fail later with a generic
# error rather than our descriptive one.
$upstreamRemote = ("$(git config "branch.$currentBranch.remote" 2>$null)").Trim()
$upstreamMerge = ("$(git config "branch.$currentBranch.merge"  2>$null)").Trim()

if (-not $upstreamRemote -or -not $upstreamMerge) {
  throw "Branch '$currentBranch' has no upstream. Set one (e.g. 'git push -u origin $currentBranch') before releasing."
}

if ($upstreamRemote -ne $MainRemote) {
  throw "Upstream for '$currentBranch' tracks '$upstreamRemote', but must track '$MainRemote' ($MainRepoSlug)."
}

# --- Fetch & ahead/behind check ---------------------------------------------

Write-Host "Fetching from '$MainRemote'..." -ForegroundColor Yellow
Invoke-Git fetch --prune --tags $MainRemote | Out-Null

$countsLine = Get-ScalarLine (Invoke-Git rev-list --left-right --count "HEAD...@{u}")
$parts = @($countsLine -split '\s+' | Where-Object { $_ -ne '' })

if ($parts.Count -lt 2) {
  throw "Failed to determine ahead/behind counts relative to upstream (output: '$countsLine')."
}

$ahead = [int]$parts[0]
$behind = [int]$parts[1]

if ($behind -gt 0) {
  throw "Branch '$currentBranch' is $behind commit(s) behind upstream. Pull or rebase before releasing."
}
if ($ahead -gt 0) {
  throw "Branch '$currentBranch' has $ahead unpushed commit(s). Push before releasing."
}

# --- Tag existence checks ----------------------------------------------------

git show-ref --tags --verify "refs/tags/$TagName" 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) {
  throw "Tag '$TagName' already exists locally."
}

$remoteTag = git ls-remote --tags $MainRemote "refs/tags/$TagName" 2>$null
if ($LASTEXITCODE -ne 0) {
  throw "Failed to query remote '$MainRemote' for tag '$TagName'."
}
if ($remoteTag) {
  throw "Tag '$TagName' already exists on remote '$MainRemote'."
}

# --- Create & push tag -------------------------------------------------------

$commitHash = Get-ScalarLine (Invoke-Git rev-parse --short HEAD)
Write-Host "Tagging commit $commitHash as $TagName (GPG-signed)..." -ForegroundColor Yellow

Invoke-Git tag --sign --annotate $TagName -m "Release $TagName"
Write-Host "Tagged $TagName locally." -ForegroundColor Green

Write-Host "Pushing to '$MainRemote'..." -ForegroundColor Yellow
Invoke-Git push $MainRemote $TagName
Write-Host "Pushed tag '$TagName' to remote." -ForegroundColor Green
