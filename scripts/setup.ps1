<#
.SYNOPSIS
Installs what a tessoa plugin project needs on this computer, and only what is
missing: the tessoa-plugin command-line tool, and Node.js (-Lang ts) or Rust
with the wasm32-wasip2 target (-Lang rust).

.DESCRIPTION
Run it from anywhere, for example:

    powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts

Every file it downloads must match the sha256 in scripts/cli.lock, whichever
server it came from; anything else is refused.

-Mirror cn takes Node.js, Rust and their packages from mirrors in mainland
China (npmmirror, rsproxy), -Mirror official never does, and the default,
auto, uses the mirrors when the official servers answer slowly or not at all.
With the mirrors, npm's registry and cargo's crates source are pointed at them
too, unless they are already set to something else.

-NoModifyPath leaves the user's PATH alone; the tools are then reachable by the
paths printed at the end.

TESSOA_PLUGIN_HOME changes where tessoa-plugin and Node.js go (default
~\.tessoa-plugin); tessoa-plugin looks there too. TESSOA_PLUGIN_DOWNLOAD
changes where tessoa-plugin is downloaded from; its sha256 still has to match.
#>
[CmdletBinding()]
param(
    [ValidateSet('', 'ts', 'rust')]
    [string] $Lang = '',
    [ValidateSet('auto', 'cn', 'official')]
    [string] $Mirror = 'auto',
    [switch] $NoModifyPath
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName System.IO.Compression.FileSystem

$Target = 'x86_64-pc-windows-msvc'
$DefaultDownload = 'https://download.tessoa.com/plugins/'
$NodeMajor = 22

function Say([string] $Text) { Write-Host "setup: $Text" }

# ---------------------------------------------------------------- cli.lock

$Versions = @{}
$Hashes = @{}
foreach ($line in Get-Content -LiteralPath (Join-Path $PSScriptRoot 'cli.lock')) {
    $f = $line.Trim() -split '\s+'
    if ($f.Count -eq 3 -and $f[0] -eq 'version') { $Versions[$f[1]] = $f[2] }
    elseif ($f.Count -eq 3 -and $f[0] -eq 'file') { $Hashes[$f[1]] = $f[2].ToLowerInvariant() }
}

if ($env:TESSOA_PLUGIN_HOME) { $Root = $env:TESSOA_PLUGIN_HOME } else { $Root = Join-Path $env:USERPROFILE '.tessoa-plugin' }
$Work = Join-Path ([IO.Path]::GetTempPath()) ('tessoa-setup-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $Work | Out-Null

# ---------------------------------------------------------------- helpers

# Download a file named in cli.lock and check its sha256 before anything uses it.
function Get-Checked([string] $Url, [string] $Name) {
    if (-not $Hashes.ContainsKey($Name)) {
        throw "scripts/cli.lock has no sha256 for $Name, so it is not downloaded"
    }
    $dest = Join-Path $Work $Name
    Say "downloading $Url"
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $dest -TimeoutSec 1800
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $dest).Hash.ToLowerInvariant()
    if ($actual -ne $Hashes[$Name]) {
        Remove-Item -Force -LiteralPath $dest
        throw "the sha256 of $Name is $actual but scripts/cli.lock says $($Hashes[$Name]); refused"
    }
    return $dest
}

function Expand-Zip([string] $Zip, [string] $Dest) {
    if (Test-Path -LiteralPath $Dest) { Remove-Item -Recurse -Force -LiteralPath $Dest }
    [IO.Compression.ZipFile]::ExtractToDirectory($Zip, $Dest)
}

# The first line a program prints for these arguments, or $null. Windows
# PowerShell turns a redirected stderr line into a terminating error under
# 'Stop', so this function relaxes that for itself.
function First-Line([string] $Exe, [string[]] $Arguments) {
    $ErrorActionPreference = 'Continue'
    try {
        $out = & $Exe @Arguments 2>$null
        if ($LASTEXITCODE -ne 0) { return $null }
        return ([string]($out | Select-Object -First 1)).Trim()
    } catch {
        return $null
    }
}

# A program on the PATH, or the first of the given files that exists.
function Find-Program([string] $Name, [string[]] $Files) {
    $c = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($c) { return $c.Source }
    foreach ($f in $Files) {
        if ($f -and (Test-Path -LiteralPath $f -PathType Leaf)) { return $f }
    }
    return $null
}

function Version-Of([string] $Text) {
    if ($Text -match '(\d+)\.(\d+)\.(\d+)') { return [version]"$($Matches[1]).$($Matches[2]).$($Matches[3])" }
    return $null
}

$script:UseMirror = $null
function Test-Quick([string] $Url) {
    try {
        $watch = [Diagnostics.Stopwatch]::StartNew()
        Invoke-WebRequest -UseBasicParsing -Uri $Url -TimeoutSec 6 | Out-Null
        return $watch.Elapsed.TotalSeconds -lt 3
    } catch {
        return $false
    }
}
function Use-Mirror {
    if ($null -ne $script:UseMirror) { return $script:UseMirror }
    if ($Mirror -eq 'cn') {
        $script:UseMirror = $true
        Say 'using the mirrors (npmmirror, rsproxy)'
    } elseif ($Mirror -eq 'official') {
        $script:UseMirror = $false
        Say 'using the official servers'
    } else {
        $fast = (Test-Quick 'https://registry.npmjs.org/-/ping') -and (Test-Quick 'https://static.rust-lang.org/rustup/release-stable.toml')
        $script:UseMirror = -not $fast
        if ($script:UseMirror) {
            Say 'the official servers are slow or out of reach from here; using the mirrors (npmmirror, rsproxy)'
        } else {
            Say 'using the official servers'
        }
    }
    return $script:UseMirror
}

# Put a folder at the front of the user's PATH, keeping the other entries
# exactly as stored (unexpanded), and tell running programs about the change.
function Add-UserPath([string] $Dir) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    try {
        $raw = [string]$key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $parts = @($raw -split ';' | Where-Object { $_ })
        if ($parts -contains $Dir) { return $false }
        $key.SetValue('Path', ((@($Dir) + $parts) -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
    } finally {
        $key.Close()
    }
    # Setting a variable through .NET broadcasts WM_SETTINGCHANGE; this one is
    # removed again at once.
    [Environment]::SetEnvironmentVariable('TESSOA_PLUGIN_SETUP', '1', 'User')
    [Environment]::SetEnvironmentVariable('TESSOA_PLUGIN_SETUP', $null, 'User')
    return $true
}

$Summary = New-Object System.Collections.ArrayList
$PathDirs = New-Object System.Collections.ArrayList
function Report([string] $Tool, [string] $Version, [string] $Path) {
    [void]$Summary.Add(('{0,-14} {1,-10} {2}' -f $Tool, $Version, $Path))
}

try {
    if (-not [Environment]::Is64BitOperatingSystem) {
        throw 'tessoa-plugin needs 64-bit Windows'
    }

    # ------------------------------------------------------------ tessoa-plugin

    $want = [version]$Versions['tessoa-plugin']
    $toolDir = Join-Path $Root 'bin'
    $toolExe = Join-Path $toolDir 'tessoa-plugin.exe'
    $found = $null
    $have = $null
    foreach ($candidate in @($toolExe, (Find-Program 'tessoa-plugin' @()))) {
        if ($candidate -and (Test-Path -LiteralPath $candidate)) {
            $v = Version-Of (First-Line $candidate @('--version'))
            if ($v -and $v -ge $want) { $found = $candidate; $have = $v; break }
        }
    }
    if ($found) {
        Say "tessoa-plugin $have found at $found"
        Report 'tessoa-plugin' "$have" $found
    } else {
        $name = "tessoa-plugin-$want-$Target.zip"
        if (-not $Hashes.ContainsKey($name)) {
            throw "this template does not pin a tessoa-plugin build for Windows yet (scripts/cli.lock has no $name)"
        }
        if ($env:TESSOA_PLUGIN_DOWNLOAD) { $base = $env:TESSOA_PLUGIN_DOWNLOAD } else { $base = $DefaultDownload }
        if (-not $base.EndsWith('/')) { $base += '/' }
        $zip = Get-Checked ($base + $name) $name
        $unpacked = Join-Path $Work 'tessoa-plugin'
        Expand-Zip $zip $unpacked
        New-Item -ItemType Directory -Force -Path $toolDir | Out-Null
        Copy-Item -Force -LiteralPath (Join-Path $unpacked 'tessoa-plugin.exe') -Destination $toolExe
        $notices = Join-Path $unpacked 'THIRD-PARTY-NOTICES.html'
        if (Test-Path -LiteralPath $notices) { Copy-Item -Force -LiteralPath $notices -Destination $toolDir }
        Say "installed tessoa-plugin $want"
        Report 'tessoa-plugin' "$want" $toolExe
        [void]$PathDirs.Add($toolDir)
    }

    # ------------------------------------------------------------ Node.js

    if ($Lang -eq 'ts') {
        $nodeWant = $Versions['node']
        $ourNodeDir = Join-Path $Root 'node'
        $ourNode = Join-Path $ourNodeDir 'node.exe'
        $systemNode = Find-Program 'node' @()
        $systemVersion = $null
        if ($systemNode) { $systemVersion = Version-Of (First-Line $systemNode @('--version')) }
        $ourVersion = $null
        if (Test-Path -LiteralPath $ourNode) { $ourVersion = Version-Of (First-Line $ourNode @('--version')) }

        if ($systemVersion -and $systemVersion.Major -ge $NodeMajor) {
            $node = $systemNode
            Say "Node.js $systemVersion found at $node"
            Report 'node' "$systemVersion" $node
        } elseif ($ourVersion -and $ourVersion.Major -ge $NodeMajor) {
            $node = $ourNode
            Say "Node.js $ourVersion found at $node"
            Report 'node' "$ourVersion" $node
        } else {
            if ($systemVersion) {
                Say "Node.js $systemVersion at $systemNode is older than $NodeMajor; installing $nodeWant for tessoa-plugin, which prefers it"
            }
            $name = "node-v$nodeWant-win-x64.zip"
            if (Use-Mirror) { $base = "https://npmmirror.com/mirrors/node/v$nodeWant/" } else { $base = "https://nodejs.org/dist/v$nodeWant/" }
            $zip = Get-Checked ($base + $name) $name
            $unpacked = Join-Path $Work 'node'
            Expand-Zip $zip $unpacked
            New-Item -ItemType Directory -Force -Path $Root | Out-Null
            if (Test-Path -LiteralPath $ourNodeDir) { Remove-Item -Recurse -Force -LiteralPath $ourNodeDir }
            Move-Item -LiteralPath (Join-Path $unpacked "node-v$nodeWant-win-x64") -Destination $ourNodeDir
            $node = $ourNode
            Say "installed Node.js $nodeWant"
            Report 'node' $nodeWant $node
            # Only when there is no other Node.js: an older one stays the
            # user's default, and tessoa-plugin finds this one by itself.
            if (-not $systemNode) { [void]$PathDirs.Add($ourNodeDir) }
        }

        $npm = Join-Path (Split-Path -Parent $node) 'npm.cmd'
        if (-not (Test-Path -LiteralPath $npm)) { $npm = Find-Program 'npm.cmd' @() }
        if (-not $npm) { throw "npm was not found next to $node" }
        Report 'npm' (First-Line $npm @('--version')) $npm
        if (Use-Mirror) {
            $registry = First-Line $npm @('config', 'get', 'registry')
            if ($registry -eq 'https://registry.npmjs.org/') {
                & $npm config set registry https://registry.npmmirror.com/ --location=user | Out-Null
                Say 'npm now uses https://registry.npmmirror.com/ (undo: npm config delete registry --location=user)'
            } elseif ($registry) {
                Say "npm already uses $registry; left as it is"
            }
        }
    }

    # ------------------------------------------------------------ Rust

    if ($Lang -eq 'rust') {
        $minRust = [version]'1.88.0'
        $sdkCargo = Join-Path $PSScriptRoot '..\sdk\rust\Cargo.toml'
        if (Test-Path -LiteralPath $sdkCargo) {
            $m = Select-String -LiteralPath $sdkCargo -Pattern '^rust-version\s*=\s*"(\d+)\.(\d+)(?:\.(\d+))?"' | Select-Object -First 1
            if ($m) {
                $g = $m.Matches[0].Groups
                $patch = 0
                if ($g[3].Success) { $patch = [int]$g[3].Value }
                $minRust = [version]"$($g[1].Value).$($g[2].Value).$patch"
            }
        }
        if ($env:CARGO_HOME) { $cargoHome = $env:CARGO_HOME } else { $cargoHome = Join-Path $env:USERPROFILE '.cargo' }
        $cargoBin = Join-Path $cargoHome 'bin'
        $cargo = Find-Program 'cargo' @((Join-Path $cargoBin 'cargo.exe'))
        $rustup = Find-Program 'rustup' @((Join-Path $cargoBin 'rustup.exe'))
        if (Use-Mirror) {
            $env:RUSTUP_DIST_SERVER = 'https://rsproxy.cn'
            $env:RUSTUP_UPDATE_ROOT = 'https://rsproxy.cn/rustup'
        }

        if (-not $cargo) {
            $v = $Versions['rustup']
            $name = "rustup-init-$v-$Target.exe"
            if (Use-Mirror) { $base = "https://rsproxy.cn/rustup/archive/$v/$Target/" } else { $base = "https://static.rust-lang.org/rustup/archive/$v/$Target/" }
            $init = Get-Checked ($base + 'rustup-init.exe') $name
            # The GNU host brings its own linker; the MSVC host would need
            # Visual Studio's. Plugins are WebAssembly either way.
            $hostTriple = 'x86_64-pc-windows-gnu'
            $initArgs = @('-y', '--profile', 'minimal', '--default-toolchain', 'stable', '--default-host', $hostTriple, '--target', 'wasm32-wasip2')
            if ($NoModifyPath) { $initArgs += '--no-modify-path' }
            Say "installing Rust (stable, $hostTriple) with rustup $v"
            & $init @initArgs
            if ($LASTEXITCODE -ne 0) { throw "rustup-init failed with exit code $LASTEXITCODE" }
            $cargo = Join-Path $cargoBin 'cargo.exe'
            $rustup = Join-Path $cargoBin 'rustup.exe'
        } else {
            $rustc = Find-Program 'rustc' @((Join-Path (Split-Path -Parent $cargo) 'rustc.exe'))
            $rustVersion = $null
            if ($rustc) { $rustVersion = Version-Of (First-Line $rustc @('--version')) }
            if (-not $rustVersion -or $rustVersion -lt $minRust) {
                if (-not $rustup) { throw "Rust $rustVersion at $cargo is older than $minRust and there is no rustup to update it" }
                Say "Rust $rustVersion is older than $minRust; updating the stable toolchain"
                & $rustup update stable
                if ($LASTEXITCODE -ne 0) { throw 'rustup update stable failed' }
            }
            if ($rustup) {
                $targets = & $rustup target list --installed
                if (-not ($targets -contains 'wasm32-wasip2')) {
                    Say 'adding the wasm32-wasip2 target'
                    & $rustup target add wasm32-wasip2
                    if ($LASTEXITCODE -ne 0) { throw 'rustup target add wasm32-wasip2 failed' }
                }
            } else {
                Say 'no rustup found; make sure the wasm32-wasip2 target is installed'
            }
        }

        $rustc = Join-Path (Split-Path -Parent $cargo) 'rustc.exe'
        Report 'cargo' "$(Version-Of (First-Line $cargo @('--version')))" $cargo
        Report 'rustc' "$(Version-Of (First-Line $rustc @('--version')))" $rustc

        if (Use-Mirror) {
            $config = Join-Path $cargoHome 'config.toml'
            $existing = ''
            if (Test-Path -LiteralPath $config) { $existing = [IO.File]::ReadAllText($config) }
            if ($existing -match '\[source\.crates-io\]') {
                Say "cargo's crates source is already set in $config; left as it is"
            } else {
                New-Item -ItemType Directory -Force -Path $cargoHome | Out-Null
                $block = "`n[source.crates-io]`nreplace-with = `"rsproxy-sparse`"`n`n[source.rsproxy-sparse]`nregistry = `"sparse+https://rsproxy.cn/index/`"`n"
                [IO.File]::AppendAllText($config, $block, (New-Object System.Text.UTF8Encoding($false)))
                Say "cargo now takes crates from rsproxy (undo: remove the [source.*] lines from $config)"
            }
        }
    }

    # ------------------------------------------------------------ PATH

    $changed = $false
    if (-not $NoModifyPath) {
        foreach ($d in $PathDirs) {
            if (Add-UserPath $d) { $changed = $true; Say "added $d to your PATH" }
        }
    }

    Write-Host ''
    foreach ($row in $Summary) { Write-Host $row }
    Write-Host ''
    if ($NoModifyPath -and $PathDirs.Count -gt 0) {
        Write-Host 'PATH was left alone (-NoModifyPath). Call the tools by the paths above.'
    } elseif ($changed) {
        Write-Host 'New terminals find these by name. A terminal opened before this setup may not;'
        Write-Host 'call the tools by the paths above until it is reopened.'
    }
    Write-Host 'Next: tessoa-plugin init (see sdk/AGENTS.md).'
    exit 0
} catch {
    Write-Host "setup: $($_.Exception.Message)"
    exit 1
} finally {
    Remove-Item -Recurse -Force -LiteralPath $Work -ErrorAction SilentlyContinue
}
