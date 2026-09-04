$path = [Environment]::GetEnvironmentVariable('PATH', 'User')
if ($path -notlike "*$env:USERPROFILE\.cargo\bin*") {
    $newPath = $path + ";$env:USERPROFILE\.cargo\bin"
    [Environment]::SetEnvironmentVariable('PATH', $newPath, 'User')
    Write-Host "Added cargo to PATH"
} else {
    Write-Host "cargo already in PATH"
}
