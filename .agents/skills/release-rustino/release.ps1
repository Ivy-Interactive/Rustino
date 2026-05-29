# release.ps1 - Automated Release Script for Rustino

$ErrorActionPreference = "Stop"

Write-Host "=== Starting Rustino Release Process ===" -ForegroundColor Cyan

# 1. Check if git workspace is clean
$status = git status --porcelain
if ($status) {
    Write-Error "Git working directory is not clean. Please commit or stash your changes before running this script."
}

# 2. Check current branch is development
$currentBranch = git branch --show-current
if ($currentBranch -ne "development") {
    Write-Host "Current branch is '$currentBranch'. Switching to 'development'..." -ForegroundColor Yellow
    git checkout development
}

# 3. Pull latest development
Write-Host "Pulling latest changes on development..." -ForegroundColor Gray
git pull origin development

# 4. Get current version and increment patch version
$propsPath = "src/Directory.Build.props"
if (-not (Test-Path $propsPath)) {
    Write-Error "Could not find Directory.Build.props at '$propsPath'."
}

$content = Get-Content $propsPath -Raw
if ($content -match '<Version>(\d+)\.(\d+)\.(\d+)(-.*)?</Version>') {
    $major = $Matches[1]
    $minor = $Matches[2]
    $patch = [int]$Matches[3]
    $suffix = $Matches[4]
    
    $newPatch = $patch + 1
    $oldVersion = "$major.$minor.$patch$suffix"
    $newVersion = "$major.$minor.$newPatch$suffix"
    
    Write-Host "Bumping version in Directory.Build.props: $oldVersion -> $newVersion" -ForegroundColor Green
    $newContent = $content -replace "<Version>$oldVersion</Version>", "<Version>$newVersion</Version>"
    Set-Content $propsPath -Value $newContent -NoNewline
} else {
    Write-Error "Could not parse version from Directory.Build.props."
}

# 5. Commit and push the version bump
Write-Host "Committing version bump..." -ForegroundColor Gray
git add $propsPath
git commit -m "chore: bump patch version to $newVersion"
Write-Host "Pushing version bump to origin/development..." -ForegroundColor Gray
git push origin development

# 6. Create PR from development to main
Write-Host "Creating Pull Request from development into main..." -ForegroundColor Gray
$prUrl = gh pr create --base main --head development --title "Release: Merge development into main (v$newVersion)" --body "Automated release PR for version $newVersion."
if (-not $prUrl) {
    Write-Error "Failed to create Pull Request."
}
Write-Host "Created PR: $prUrl" -ForegroundColor Green

# 7. Post the required comment for Claude review (user rule)
Write-Host "Requesting Claude review on the PR..." -ForegroundColor Gray
gh pr comment $prUrl --body "@claude review this"

# 8. Check mergeability and wait if needed
Write-Host "Waiting for PR mergeability checks..." -ForegroundColor Gray
$attempts = 0
$maxAttempts = 12 # 60 seconds total
$isMergeable = $false

while ($attempts -lt $maxAttempts) {
    Start-Sleep -Seconds 5
    $prInfo = gh pr view $prUrl --json mergeable | ConvertFrom-Json
    $mergeableState = $prInfo.mergeable
    
    Write-Host "Mergeability state: $mergeableState" -ForegroundColor Gray
    
    if ($mergeableState -eq "MERGEABLE") {
        $isMergeable = $true
        break
    } elseif ($mergeableState -eq "CONFLICTING") {
        Write-Error "PR has merge conflicts! Please resolve conflicts manually and finish the release."
    }
    
    $attempts++
}

if (-not $isMergeable) {
    Write-Error "PR mergeability check timed out or is not mergeable. Please merge manually."
}

# 9. Merge development into main
Write-Host "Merging PR into main..." -ForegroundColor Gray
gh pr merge $prUrl --merge --delete-branch=$false
Write-Host "Successfully merged PR into main!" -ForegroundColor Green

# 10. Sync main back into development
Write-Host "Synchronizing branches (merging main back into development)..." -ForegroundColor Gray
git checkout main
git pull origin main
git checkout development
git merge main --no-ff -m "Merge branch 'main' into development to sync [skip ci]"
git push origin development

# 11. Trigger release
Write-Host "Triggering NuGet release workflow on main branch..." -ForegroundColor Gray
gh workflow run release.yml --ref main
Start-Sleep -Seconds 3
gh run list --workflow=release.yml --limit 1

Write-Host "=== Rustino Release Process Completed Successfully ===" -ForegroundColor Green
