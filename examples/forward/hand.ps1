<#
.SYNOPSIS
    Does with the pointer, the keys and Windows' window calls what the forward example's steps ask, one after
    another, and writes what Windows says of the windows where a step asks.

.DESCRIPTION
    Run on the signed-in desktop under Windows PowerShell 5.1, which a clean Windows 11 has, since UI Automation's
    managed client is a .NET Framework assembly. A window is named by its process's name or number. Each line of the
    steps file is one step:

      front PROCESS       brings the process's window to the front, restored where it is minimised, so that the
                          steps after it fall on it
      press NAME          the left button pressed and released at the middle of the element of the front window that
                          UI Automation names NAME
      keys TEXT           keys as System.Windows.Forms.SendKeys writes them, to the window in front
      wait MS             a pause
      minimise PROCESS    minimises the process's window, as its minimise button would
      allow PROCESS       hands the process the right to take the foreground, and writes whether Windows took it
      state PROCESS       writes whether the process's window is minimised, maximised or restored, and where it
                          stands when restored
      foreground PROCESS  writes whether the window in front is the process's
      open PROGRAM        starts a program and brings its window to the front
      close PROGRAM       ends the program the step before opened

.PARAMETER Steps
    The file of steps, one to a line.

.PARAMETER Out
    The file the answers are written to, one line for each step that asks.
#>
[CmdletBinding()]
[OutputType([void])]
param(
    [Parameter(Mandatory)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string] $Steps,

    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string] $Out
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
Add-Type -Namespace Pamba -Name Hand -MemberDefinition @'
[StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left; public int Top; public int Right; public int Bottom; }
[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr window);
[DllImport("user32.dll")] public static extern System.IntPtr GetForegroundWindow();
[DllImport("user32.dll")] public static extern bool AllowSetForegroundWindow(uint process);
[DllImport("user32.dll")] public static extern bool ShowWindow(System.IntPtr window, int show);
[DllImport("user32.dll")] public static extern bool IsIconic(System.IntPtr window);
[DllImport("user32.dll")] public static extern bool IsZoomed(System.IntPtr window);
[DllImport("user32.dll")] public static extern bool GetWindowRect(System.IntPtr window, out Rect rect);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr window, out uint process);
[DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, System.UIntPtr extra);
'@

$moveTo = 0x8001
$leftDown = 0x0002
$leftUp = 0x0004
# ShowWindow's ask to minimise a window and activate the next one, as the minimise button does.
$minimise = 6
# ShowWindow's ask to restore a minimised window to the size and place it had.
$restore = 9
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds

function Move-Pointer {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][int] $X, [Parameter(Mandatory)][int] $Y)
    process {
        # A move in the units of real input, 0 to 65535 across the screen, so that a program sees a pointer that
        # travelled and not one that was set down.
        $across = [uint32][Math]::Round($X * 65535 / ($screen.Width - 1))
        $down = [uint32][Math]::Round($Y * 65535 / ($screen.Height - 1))
        [Pamba.Hand]::mouse_event($moveTo, $across, $down, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 30
    }
}

function Send-LeftButton {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][bool] $Down)
    process {
        $flag = if ($Down) { $leftDown } else { $leftUp }
        [Pamba.Hand]::mouse_event($flag, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 40
    }
}

function Get-Window {
    [CmdletBinding()]
    [OutputType([System.Diagnostics.Process])]
    param([Parameter(Mandatory)][string] $Name)
    process {
        # A window is found once, while it stands as Windows first shows it: a minimised window's process may no
        # longer name it as its main one.
        if (-not $script:found.ContainsKey($Name)) {
            $process = if ($Name -match '^\d+$') { Get-Process -Id ([int] $Name) } else { Get-Process -Name $Name | Select-Object -First 1 }
            if ($process.MainWindowHandle -eq [IntPtr]::Zero) { throw "process $Name has no window" }
            $script:found[$Name] = $process
        }
        $script:found[$Name]
    }
}

function Find-Named {
    [CmdletBinding()]
    [OutputType([System.Windows.Automation.AutomationElement])]
    param([Parameter(Mandatory)][string] $Name)
    process {
        if ($null -eq $script:front) { throw "a step names $Name before any step brought a window to the front" }
        $named = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty, $Name)
        $found = $script:front.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $named)
        if ($null -eq $found) { throw "the front window holds nothing named $Name" }
        $found
    }
}

[void][Pamba.Hand]::SetProcessDPIAware()
$script:front = $null
$script:found = @{}
$opened = @{}
$answers = [System.Collections.Generic.List[string]]::new()

foreach ($line in Get-Content -LiteralPath $Steps) {
    $step = $line.Trim()
    if ($step.Length -eq 0) { continue }
    $verb, $rest = $step -split '\s+', 2
    switch ($verb) {
        'front' {
            $window = (Get-Window -Name $rest).MainWindowHandle
            if ([Pamba.Hand]::IsIconic($window)) { [void][Pamba.Hand]::ShowWindow($window, $restore) }
            [void][Pamba.Hand]::SetForegroundWindow($window)
            $script:front = [System.Windows.Automation.AutomationElement]::FromHandle($window)
            Start-Sleep -Milliseconds 400
        }
        'press' {
            $box = (Find-Named -Name $rest).Current.BoundingRectangle
            if ($box.IsEmpty -or $box.Right -le 0 -or $box.Bottom -le 0) { throw "$rest stands nowhere on the screen" }
            Move-Pointer -X ([int]($box.X + $box.Width / 2)) -Y ([int]($box.Y + $box.Height / 2))
            Send-LeftButton -Down $true
            Send-LeftButton -Down $false
        }
        'keys' {
            [System.Windows.Forms.SendKeys]::SendWait($rest)
            Start-Sleep -Milliseconds 120
        }
        'wait' { Start-Sleep -Milliseconds ([int] $rest) }
        'minimise' {
            [void][Pamba.Hand]::ShowWindow((Get-Window -Name $rest).MainWindowHandle, $minimise)
            Start-Sleep -Milliseconds 400
        }
        'allow' {
            $taken = [Pamba.Hand]::AllowSetForegroundWindow([uint32](Get-Window -Name $rest).Id)
            $answers.Add("allow ${rest}: $taken")
        }
        'state' {
            $window = (Get-Window -Name $rest).MainWindowHandle
            $now = if ([Pamba.Hand]::IsIconic($window)) {
                'minimised'
            } elseif ([Pamba.Hand]::IsZoomed($window)) {
                'maximised'
            } else {
                $box = [Pamba.Hand+Rect]::new()
                [void][Pamba.Hand]::GetWindowRect($window, [ref] $box)
                "restored at $($box.Left) $($box.Top) $($box.Right) $($box.Bottom)"
            }
            $answers.Add("state ${rest}: $now")
        }
        'foreground' {
            $owner = [uint32] 0
            [void][Pamba.Hand]::GetWindowThreadProcessId([Pamba.Hand]::GetForegroundWindow(), [ref] $owner)
            $answers.Add("foreground ${rest}: $($owner -eq (Get-Window -Name $rest).Id)")
        }
        'open' {
            $process = Start-Process -FilePath $rest -PassThru
            $waited = 0
            while ($process.MainWindowHandle -eq [IntPtr]::Zero) {
                if ($waited -ge 10000) { throw "$rest showed no window in 10 s" }
                Start-Sleep -Milliseconds 200
                $waited += 200
                $process.Refresh()
            }
            $opened[$rest] = $process
            $script:found[$process.ProcessName] = $process
            [void][Pamba.Hand]::SetForegroundWindow($process.MainWindowHandle)
            $script:front = [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
            Start-Sleep -Milliseconds 400
        }
        'close' {
            if (-not $opened.ContainsKey($rest)) { throw "a step closes $rest, which no step opened" }
            Stop-Process -Id $opened[$rest].Id
            $opened.Remove($rest)
            $script:front = $null
        }
        default { throw "a step this script does not know: $step" }
    }
}

Set-Content -LiteralPath $Out -Value $answers -Encoding utf8
