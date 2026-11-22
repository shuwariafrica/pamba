<#
.SYNOPSIS
    Does with a pointer, a finger, a pen, keys and UI Automation what the presses example takes facts of, one step
    after another, and writes each act into the example's own log with Windows' tick count, so that the two read as
    one timeline.

.DESCRIPTION
    Run on the signed-in desktop under Windows PowerShell 5.1, which a clean Windows 11 has, since UI Automation's
    managed client is a .NET Framework assembly. It is aware of each screen's own scale, so places are in the screen's
    pixels at the scale it shows now. A control is found by UI Automation once and its middle remembered: a step on
    it while the window's thread is stopped clicks the remembered place and asks the window nothing. Keys are sent as
    Windows' keyboard input, which returns at once whatever the window is doing. Each line of the steps file is one
    step:

      front PROCESS      brings the first window of a process, by its name, to the front
      find NAME          remembers the middle of the first element of the front window whose name begins with NAME
      click NAME         the left button pressed and released at NAME's remembered middle
      twice NAME MS      two such clicks, MS milliseconds from the first's release to the second's press
      key KEY            a key pressed and let go: Enter, Space, Tab or Shift
      chord KEY1 KEY2    KEY2 pressed and let go while KEY1 is held
      hold KEY MS        a key held MS milliseconds; Windows repeats no injected key, so the hand sends the repeats
                         at Windows' keyboard delay and speed
      tap NAME           a finger down and up at NAME's middle
      pen NAME           a pen down and up at NAME's middle
      invoke NAME        UI Automation's Invoke on NAME
      focus              writes the name, kind and class of the control that holds the keyboard
      click-time MS      sets the double-click time as the person's own setting is set, and writes it read back
      load SECONDS       starts Use-Processor.ps1 beside this script for that long, and writes how many workers it
                         started once they have
      loaded             writes how many of the load's workers are counting now
      unload             waits for the load to end, and writes whether it ended and how many workers are left
      mark TEXT          writes TEXT
      wait MS            a pause

.PARAMETER Steps
    The file of steps, one to a line.

.PARAMETER Log
    The example's log, to which each act is appended.

.PARAMETER Out
    The file the answers are written to: each focus, each mark, and each act that failed.
#>
[CmdletBinding()]
[OutputType([void])]
param(
    [Parameter(Mandatory)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string] $Steps,

    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string] $Log,

    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string] $Out
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms
Add-Type -Namespace Pamba -Name Facts -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(System.IntPtr context);
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr window);
[DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, System.UIntPtr extra);
[DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, System.UIntPtr extra);
[DllImport("user32.dll")] public static extern uint MapVirtualKey(uint code, uint kind);
[DllImport("user32.dll")] public static extern uint GetDoubleClickTime();
[DllImport("user32.dll", SetLastError = true)] public static extern bool SystemParametersInfo(uint action, uint value, System.IntPtr data, uint flags);
[DllImport("kernel32.dll")] public static extern ulong GetTickCount64();

[StructLayout(LayoutKind.Sequential)] public struct Point { public int X; public int Y; }
[StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left; public int Top; public int Right; public int Bottom; }
[StructLayout(LayoutKind.Sequential)] public struct PointerInfo {
    public uint Kind; public uint Id; public uint Frame; public uint Flags;
    public System.IntPtr Device; public System.IntPtr Target;
    public Point Pixel; public Point Himetric; public Point PixelRaw; public Point HimetricRaw;
    public uint Time; public uint History; public int Input; public uint Keys; public ulong Performance; public int Button;
}
[StructLayout(LayoutKind.Sequential)] public struct TouchInfo {
    public PointerInfo Pointer; public uint TouchFlags; public uint TouchMask;
    public Rect Contact; public Rect ContactRaw; public uint Orientation; public uint Pressure;
}
[StructLayout(LayoutKind.Sequential)] public struct PenInfo {
    public PointerInfo Pointer; public uint PenFlags; public uint PenMask;
    public uint Pressure; public uint Rotation; public int TiltX; public int TiltY;
}
// POINTER_TYPE_INFO: the kind, then the larger of a touch's and a pen's information.
[StructLayout(LayoutKind.Sequential, Size = 152)] public struct PenTypeInfo { public uint Kind; public PenInfo Pen; }
[DllImport("user32.dll", SetLastError = true)] public static extern bool InitializeTouchInjection(uint most, uint mode);
[DllImport("user32.dll", SetLastError = true)] public static extern bool InjectTouchInput(uint count, [In] TouchInfo[] contacts);
[DllImport("user32.dll", SetLastError = true)] public static extern System.IntPtr CreateSyntheticPointerDevice(uint kind, uint most, uint feedback);
[DllImport("user32.dll", SetLastError = true)] public static extern bool InjectSyntheticPointerInput(System.IntPtr device, [In] PenTypeInfo[] input, uint count);

public static bool Finger(int x, int y, uint flags) {
    TouchInfo[] contact = new TouchInfo[1];
    contact[0].Pointer.Kind = 2;
    contact[0].Pointer.Flags = flags;
    contact[0].Pointer.Pixel.X = x;
    contact[0].Pointer.Pixel.Y = y;
    contact[0].TouchMask = 0x7;
    contact[0].Orientation = 90;
    contact[0].Pressure = 32000;
    contact[0].Contact.Left = x - 2; contact[0].Contact.Right = x + 2;
    contact[0].Contact.Top = y - 2; contact[0].Contact.Bottom = y + 2;
    return InjectTouchInput(1, contact);
}

public static bool Pen(System.IntPtr device, int x, int y, uint flags) {
    PenTypeInfo[] input = new PenTypeInfo[1];
    input[0].Kind = 3;
    input[0].Pen.Pointer.Kind = 3;
    input[0].Pen.Pointer.Flags = flags;
    input[0].Pen.Pointer.Pixel.X = x;
    input[0].Pen.Pointer.Pixel.Y = y;
    input[0].Pen.PenMask = 0x1;
    input[0].Pen.Pressure = 512;
    return InjectSyntheticPointerInput(device, input, 1);
}
'@

# Aware of each screen's own scale (-4 is that awareness, version 2), so that UI Automation's places, the screen's
# size and the pointer's moves are in one set of pixels.
[void][Pamba.Facts]::SetProcessDpiAwarenessContext([IntPtr] -4)
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$keyNames = @{ Enter = 0x0D; Space = 0x20; Tab = 0x09; Shift = 0x10 }
$script:log = $Log
$answers = [System.Collections.Generic.List[string]]::new()
$places = @{}
$script:front = $null
$script:fingers = $false
$script:pen = [IntPtr]::Zero
$script:load = $null
$script:loadIds = @()

function Write-Act {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][string] $Text)
    process {
        # Opened for appending alone, so that Windows puts each write at the end the program's own writes have
        # reached: Add-Content writes at the end it saw on opening and can overwrite a line the program wrote since.
        $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes(
            "hand tick=$([Pamba.Facts]::GetTickCount64()) $Text`n")
        $stream = [System.IO.FileStream]::new($script:log, [System.IO.FileMode]::Append,
            [System.Security.AccessControl.FileSystemRights]::AppendData, [System.IO.FileShare]::ReadWrite, 4096,
            [System.IO.FileOptions]::None)
        try {
            $stream.Write($bytes, 0, $bytes.Length)
        } finally {
            $stream.Dispose()
        }
    }
}

function Get-CountingWorker {
    [CmdletBinding()]
    [OutputType([int])]
    param()
    process {
        # A worker that has ended is no longer a process: its absence is the answer, not a fault.
        @($script:loadIds | Where-Object { Get-Process -Id $_ -ErrorAction Ignore }).Count
    }
}

function Move-Pointer {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][int] $X, [Parameter(Mandatory)][int] $Y)
    process {
        $across = [uint32][Math]::Round($X * 65535 / ($screen.Width - 1))
        $down = [uint32][Math]::Round($Y * 65535 / ($screen.Height - 1))
        [Pamba.Facts]::mouse_event(0x8001, $across, $down, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 30
    }
}

function Send-Click {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][string] $Name)
    process {
        $at = Get-Place -Name $Name
        Move-Pointer -X $at.X -Y $at.Y
        Write-Act -Text "left button down on $Name"
        [Pamba.Facts]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 40
        [Pamba.Facts]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
        Write-Act -Text "left button up on $Name"
    }
}

function Find-Place {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][string] $Name)
    process {
        if ($null -eq $script:front) { throw "a front step must come before the step that finds $Name" }
        $all = $script:front.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
        foreach ($element in $all) {
            if ($element.Current.Name.StartsWith($Name, [System.StringComparison]::Ordinal)) {
                $bounds = $element.Current.BoundingRectangle
                $places[$Name] = [pscustomobject]@{
                    X       = [int]($bounds.Left + $bounds.Width / 2)
                    Y       = [int]($bounds.Top + $bounds.Height / 2)
                    Element = $element
                }
                return
            }
        }
        throw "nothing in the front window has a name that begins with `"$Name`""
    }
}

function Get-Place {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)][string] $Name)
    process {
        if (-not $places.ContainsKey($Name)) { Find-Place -Name $Name }
        $places[$Name]
    }
}

function Send-Key {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][string] $Key, [Parameter(Mandatory)][bool] $Down)
    process {
        if (-not $keyNames.ContainsKey($Key)) { throw "a key this script does not know: $Key" }
        $code = [byte] $keyNames[$Key]
        $scan = [byte] [Pamba.Facts]::MapVirtualKey($code, 0)
        $flags = if ($Down) { 0 } else { 2 }
        [Pamba.Facts]::keybd_event($code, $scan, $flags, [UIntPtr]::Zero)
        Write-Act -Text "$Key $(if ($Down) { 'down' } else { 'up' })"
    }
}

function Get-Focus {
    [CmdletBinding()]
    [OutputType([string])]
    param()
    process {
        $held = [Windows.Automation.AutomationElement]::FocusedElement
        if ($null -eq $held) { return 'focus: nothing' }
        $now = $held.Current
        "focus: `"$($now.Name)`" $($now.ControlType.ProgrammaticName) $($now.ClassName)"
    }
}

foreach ($line in Get-Content -LiteralPath $Steps) {
    $step = $line.Trim()
    if ($step.Length -eq 0) { continue }
    $verb, $rest = $step -split '\s+', 2
    try {
        switch ($verb) {
            'front' {
                $process = Get-Process -Name $rest | Select-Object -First 1
                $window = $process.MainWindowHandle
                if ($window -eq [IntPtr]::Zero) { throw "process $rest has no window to bring to the front" }
                [void][Pamba.Facts]::SetForegroundWindow($window)
                $script:front = [Windows.Automation.AutomationElement]::FromHandle($window)
                Start-Sleep -Milliseconds 400
            }
            'find' { Find-Place -Name $rest }
            'click' { Send-Click -Name $rest }
            'twice' {
                $name, $gap = $rest -split '\s+(?=\d+$)'
                Send-Click -Name $name
                Start-Sleep -Milliseconds ([int] $gap)
                Send-Click -Name $name
            }
            'key' {
                Send-Key -Key $rest -Down $true
                Start-Sleep -Milliseconds 30
                Send-Key -Key $rest -Down $false
            }
            'chord' {
                $held, $pressed = $rest -split '\s+'
                Send-Key -Key $held -Down $true
                Send-Key -Key $pressed -Down $true
                Start-Sleep -Milliseconds 30
                Send-Key -Key $pressed -Down $false
                Send-Key -Key $held -Down $false
            }
            'hold' {
                $key, $length = $rest -split '\s+'
                # KeyboardDelay 0 to 3 is a quarter of a second to one second; KeyboardSpeed 0 to 31 is about 2.5 to
                # 30 repeats a second.
                $delay = 250 * ([System.Windows.Forms.SystemInformation]::KeyboardDelay + 1)
                $rate = 2.5 + [System.Windows.Forms.SystemInformation]::KeyboardSpeed * 27.5 / 31
                $every = [int](1000 / $rate)
                Write-Act -Text "hold $key for $length ms: repeats after $delay ms, every $every ms"
                $started = [Pamba.Facts]::GetTickCount64()
                Send-Key -Key $key -Down $true
                Start-Sleep -Milliseconds $delay
                while (([Pamba.Facts]::GetTickCount64() - $started) -lt [int] $length) {
                    Send-Key -Key $key -Down $true
                    Start-Sleep -Milliseconds $every
                }
                Send-Key -Key $key -Down $false
            }
            'tap' {
                $at = Get-Place -Name $rest
                if (-not $script:fingers) {
                    if (-not [Pamba.Facts]::InitializeTouchInjection(1, 1)) { throw 'Windows refused to let a finger be injected' }
                    $script:fingers = $true
                }
                Write-Act -Text "finger down on $rest"
                if (-not [Pamba.Facts]::Finger($at.X, $at.Y, 0x10006)) { throw "the finger did not go down on $rest" }
                Start-Sleep -Milliseconds 60
                if (-not [Pamba.Facts]::Finger($at.X, $at.Y, 0x40000)) { throw "the finger did not come up on $rest" }
                Write-Act -Text "finger up on $rest"
            }
            'pen' {
                $at = Get-Place -Name $rest
                if ($script:pen -eq [IntPtr]::Zero) {
                    $script:pen = [Pamba.Facts]::CreateSyntheticPointerDevice(3, 1, 1)
                    if ($script:pen -eq [IntPtr]::Zero) { throw "Windows refused a pen: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())" }
                }
                # In range above the place, down, up, and out of range again.
                foreach ($flags in 0x20002, 0x10006, 0x40002, 0x20000) {
                    if (-not [Pamba.Facts]::Pen($script:pen, $at.X, $at.Y, [uint32] $flags)) {
                        throw "the pen's input 0x$('{0:X}' -f $flags) was refused: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
                    }
                    Write-Act -Text "pen 0x$('{0:X}' -f $flags) on $rest"
                    Start-Sleep -Milliseconds 40
                }
            }
            'invoke' {
                $at = Get-Place -Name $rest
                Write-Act -Text "invoke $rest"
                $at.Element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
                Write-Act -Text "invoked $rest"
            }
            'focus' {
                $held = Get-Focus
                $answers.Add($held)
                Write-Act -Text $held
            }
            'click-time' {
                # SPI_SETDOUBLECLICKTIME, written to the person's profile and announced, as the mouse settings write it.
                if (-not [Pamba.Facts]::SystemParametersInfo(0x0020, [uint32] $rest, [IntPtr]::Zero, 3)) {
                    throw "Windows refused the double-click time $rest"
                }
                $read = "double-click time set $rest, read back $([Pamba.Facts]::GetDoubleClickTime())"
                $answers.Add($read)
                Write-Act -Text $read
            }
            'load' {
                $script = Join-Path $PSScriptRoot 'Use-Processor.ps1'
                $idFile = Join-Path ([System.IO.Path]::GetTempPath()) "presses-load-$PID.txt"
                # An earlier run's file is the only one there can be, and its absence is no fault.
                Remove-Item -LiteralPath $idFile -ErrorAction Ignore
                $script:load = Start-Process -FilePath powershell -WindowStyle Hidden -PassThru -ArgumentList '-NoProfile',
                    '-ExecutionPolicy', 'Bypass', '-File', $script, '-Seconds', $rest, '-PidFile', $idFile
                # Use-Processor.ps1 starts one worker for each logical processor and writes their ids once all have
                # started; reading until all are there keeps a half-written file from being counted.
                $script:loadIds = @()
                $until = [DateTime]::UtcNow.AddSeconds(15)
                while ($script:loadIds.Count -lt [Environment]::ProcessorCount -and [DateTime]::UtcNow -lt $until) {
                    Start-Sleep -Milliseconds 100
                    if (Test-Path -LiteralPath $idFile) {
                        $script:loadIds = @(Get-Content -LiteralPath $idFile | Where-Object { $_ } | ForEach-Object { [int] $_ })
                    }
                }
                Write-Act -Text "processors loaded by $($script:loadIds.Count) workers for $rest s"
            }
            'loaded' {
                $read = "load: $(Get-CountingWorker) of $($script:loadIds.Count) workers counting"
                $answers.Add($read)
                Write-Act -Text $read
            }
            'unload' {
                if ($null -eq $script:load) { throw 'no load was started' }
                $read = "load ended $($script:load.WaitForExit(60000)), workers left $(Get-CountingWorker)"
                $script:load = $null
                $answers.Add($read)
                Write-Act -Text $read
            }
            'mark' {
                $answers.Add("mark: $rest")
                Write-Act -Text "mark: $rest"
            }
            'wait' { Start-Sleep -Milliseconds ([int] $rest) }
            default { throw "a step this script does not know: $step" }
        }
    }
    catch [System.Management.Automation.RuntimeException] {
        $answers.Add("failed: $step - $($_.Exception.Message)")
        Write-Act -Text "failed: $step - $($_.Exception.Message)"
    }
}

if ($null -ne $script:load) {
    $read = "load ended $($script:load.WaitForExit(60000)), workers left $(Get-CountingWorker)"
    $answers.Add($read)
    Write-Act -Text $read
}
Set-Content -LiteralPath $Out -Value $answers -Encoding utf8
