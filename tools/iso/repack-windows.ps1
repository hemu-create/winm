#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$IsoPath,

    [Parameter(Mandatory = $true)]
    [string]$OutputIso,

    [string]$DriverDir,

    [string]$UnattendPath,

    [int]$ImageIndex = 1,

    [string]$Oscdimg = "oscdimg.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $IsoPath)) {
    throw "ISO not found: $IsoPath"
}

if ($DriverDir -and -not (Test-Path -LiteralPath $DriverDir)) {
    throw "Driver directory not found: $DriverDir"
}

if ($UnattendPath -and -not (Test-Path -LiteralPath $UnattendPath)) {
    throw "Unattend file not found: $UnattendPath"
}

$mountedIso = $null
$root = Join-Path $env:TEMP ("winm-iso-" + [guid]::NewGuid().ToString("N"))

try {
    $mountedIso = Mount-DiskImage -ImagePath $IsoPath -PassThru
    $drive = (Get-Volume -DiskImage $mountedIso).DriveLetter
    if (-not $drive) {
        throw "Mounted ISO has no drive letter"
    }

    New-Item -ItemType Directory -Path $root | Out-Null
    $copy = Start-Process -FilePath "robocopy.exe" -ArgumentList @("$drive`:\", $root, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/NP") -Wait -PassThru
    if ($copy.ExitCode -ge 8) {
        throw "robocopy failed with exit code $($copy.ExitCode)"
    }

    $efiBoot = Join-Path $root "efi\microsoft\boot\efisys.bin"
    if (-not (Test-Path -LiteralPath $efiBoot)) {
        $efiBoot = Join-Path $root "boot\efisys.bin"
    }
    if (-not (Test-Path -LiteralPath $efiBoot)) {
        throw "Could not find efisys.bin in the ISO"
    }

    $installWim = Join-Path $root "sources\install.wim"
    $bootWim = Join-Path $root "sources\boot.wim"
    if (-not (Test-Path -LiteralPath $installWim)) {
        throw "install.wim not found: $installWim"
    }
    if (-not (Test-Path -LiteralPath $bootWim)) {
        throw "boot.wim not found: $bootWim"
    }

    $mount = Join-Path $env:TEMP ("winm-wim-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $mount | Out-Null

    $installInfo = Get-WindowsImage -ImagePath $installWim | Where-Object ImageIndex -eq $ImageIndex
    if (-not $installInfo) {
        throw "install.wim has no image index $ImageIndex"
    }

    Mount-WindowsImage -ImagePath $installWim -Index $ImageIndex -Path $mount | Out-Null
    if ($DriverDir) {
        Add-WindowsDriver -Path $mount -Driver $DriverDir -Recurse -ForceUnsigned | Out-Null
    }
    if ($UnattendPath) {
        $sysprep = Join-Path $mount "Windows\System32\Sysprep"
        New-Item -ItemType Directory -Path $sysprep -Force | Out-Null
        Copy-Item -LiteralPath $UnattendPath -Destination (Join-Path $sysprep "unattend.xml") -Force
    }
    Dismount-WindowsImage -Path $mount -Save | Out-Null
    Remove-Item -LiteralPath $mount -Recurse -Force

    $bootImages = Get-WindowsImage -ImagePath $bootWim
    foreach ($bootImage in $bootImages) {
        New-Item -ItemType Directory -Path $mount | Out-Null
        Mount-WindowsImage -ImagePath $bootWim -Index $bootImage.ImageIndex -Path $mount | Out-Null
        if ($DriverDir) {
            Add-WindowsDriver -Path $mount -Driver $DriverDir -Recurse -ForceUnsigned | Out-Null
        }
        if ($UnattendPath) {
            Copy-Item -LiteralPath $UnattendPath -Destination (Join-Path $root "autounattend.xml") -Force
        }
        Dismount-WindowsImage -Path $mount -Save | Out-Null
        Remove-Item -LiteralPath $mount -Recurse -Force
    }

    if ($UnattendPath) {
        Copy-Item -LiteralPath $UnattendPath -Destination (Join-Path $root "autounattend.xml") -Force
    }

    if (-not (Get-Command $Oscdimg -ErrorAction SilentlyContinue)) {
        throw "oscdimg.exe not found. Install the Windows ADK Deployment Tools or pass -Oscdimg."
    }

    & $Oscdimg -m -o -u2 -udfver102 "-bootdata:1#pEF,e,b$efiBoot" $root $OutputIso
    if ($LASTEXITCODE -ne 0) {
        throw "oscdimg failed with exit code $LASTEXITCODE"
    }

    Write-Host "Wrote $OutputIso"
}
finally {
    if ($mountedIso) {
        Dismount-DiskImage -ImagePath $IsoPath | Out-Null
    }
    if (Test-Path -LiteralPath $root) {
        Remove-Item -LiteralPath $root -Recurse -Force
    }
}
