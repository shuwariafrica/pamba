<#
.SYNOPSIS
    Does with the pointer and the keys what a person at the machine would, one step after another, and writes what
    the clipboard and the keyboard's focus hold where a step asks.

.DESCRIPTION
    Run on the signed-in desktop: a session with no desktop has no pointer. It runs under Windows PowerShell 5.1,
    which a clean Windows 11 has, since UI Automation's managed client is a .NET Framework assembly. Places are in
    the screen's own pixels. Each line of the steps file is one step:

      front PROCESS       brings the first window of a process, by its number or its name, to the front, so that the
                          steps after it fall on it
      move X Y            the pointer to a place
      click X Y           the left button pressed and released there
      double X Y          twice, as a double click
      drag X1 Y1 X2 Y2    the left button down at the first place, across in twenty moves, up at the second
      keys TEXT           keys as System.Windows.Forms.SendKeys writes them: ^c is Ctrl+C, {TAB} is Tab; {SPACE} is the
                          space bar
      wait MS             a pause
      clear               empties the clipboard
      clipboard           writes the clipboard's text
      cursor              writes which of Windows' pointer shapes is shown: arrow, text, west-east and so on
      touch X1 Y1 X2 Y2   a finger down at the first place, across in twenty moves, up at the second
      focus               writes the name, kind and class of the control that holds the keyboard

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
[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr window);
[DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, System.UIntPtr extra);
[DllImport("user32.dll")] public static extern System.IntPtr LoadCursor(System.IntPtr module, System.IntPtr name);

[StructLayout(LayoutKind.Sequential)] public struct Point { public int X; public int Y; }
[StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left; public int Top; public int Right; public int Bottom; }
[StructLayout(LayoutKind.Sequential)] public struct CursorInfo { public int Size; public int Flags; public System.IntPtr Cursor; public Point At; }
[DllImport("user32.dll")] public static extern bool GetCursorInfo(ref CursorInfo info);

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
[DllImport("user32.dll", SetLastError = true)] public static extern bool InitializeTouchInjection(uint most, uint mode);
[DllImport("user32.dll", SetLastError = true)] public static extern bool InjectTouchInput(uint count, [In] TouchInfo[] contacts);

public static string Cursor() {
    CursorInfo info = new CursorInfo();
    info.Size = Marshal.SizeOf(typeof(CursorInfo));
    if (!GetCursorInfo(ref info)) { return "unknown"; }
    string[] names = { "arrow", "text", "wait", "cross", "up", "northwest-southeast", "northeast-southwest", "west-east", "north-south", "all", "no", "hand", "starting", "help" };
    int[] ids = { 32512, 32513, 32514, 32515, 32516, 32642, 32643, 32644, 32645, 32646, 32648, 32649, 32650, 32651 };
    for (int at = 0; at < ids.Length; at++) {
        if (LoadCursor(System.IntPtr.Zero, new System.IntPtr(ids[at])) == info.Cursor) { return names[at]; }
    }
    return "another shape (" + info.Cursor.ToString() + ")";
}

// One finger at a place: flags 0x10006 is down, 0x20006 a move in contact, 0x40000 up.
public static bool Finger(int x, int y, uint flags) {
    TouchInfo[] contact = new TouchInfo[1];
    contact[0].Pointer.Kind = 2;
    contact[0].Pointer.Id = 0;
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
'@

$moveTo = 0x8001
$leftDown = 0x0002
$leftUp = 0x0004
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

function Set-LeftButton {
    [CmdletBinding()]
    [OutputType([void])]
    param([Parameter(Mandatory)][bool] $Down)
    process {
        $flag = if ($Down) { $leftDown } else { $leftUp }
        [Pamba.Hand]::mouse_event($flag, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 40
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

[void][Pamba.Hand]::SetProcessDPIAware()
$script:fingers = $false
$answers = [System.Collections.Generic.List[string]]::new()

foreach ($line in Get-Content -LiteralPath $Steps) {
    $step = $line.Trim()
    if ($step.Length -eq 0) { continue }
    $verb, $rest = $step -split '\s+', 2
    $places = if ($null -ne $rest -and $verb -in 'move', 'click', 'double', 'drag') { $rest -split '\s+' | ForEach-Object { [int] $_ } } else { @() }
    switch ($verb) {
        'front' {
            $process = if ($rest -match '^\d+$') { Get-Process -Id ([int] $rest) } else { Get-Process -Name $rest | Select-Object -First 1 }
            $window = $process.MainWindowHandle
            if ($window -eq [IntPtr]::Zero) { throw "process $rest has no window to bring to the front" }
            [void][Pamba.Hand]::SetForegroundWindow($window)
            Start-Sleep -Milliseconds 400
        }
        'move' { Move-Pointer -X $places[0] -Y $places[1] }
        'click' {
            Move-Pointer -X $places[0] -Y $places[1]
            Set-LeftButton -Down $true
            Set-LeftButton -Down $false
        }
        'double' {
            Move-Pointer -X $places[0] -Y $places[1]
            foreach ($press in 1..2) {
                Set-LeftButton -Down $true
                Set-LeftButton -Down $false
            }
        }
        'drag' {
            Move-Pointer -X $places[0] -Y $places[1]
            Set-LeftButton -Down $true
            foreach ($part in 1..20) {
                $x = $places[0] + [int](($places[2] - $places[0]) * $part / 20)
                $y = $places[1] + [int](($places[3] - $places[1]) * $part / 20)
                Move-Pointer -X $x -Y $y
            }
            Set-LeftButton -Down $false
        }
        'keys' {
            [System.Windows.Forms.SendKeys]::SendWait($rest.Replace('{SPACE}', ' '))
            Start-Sleep -Milliseconds 120
        }
        'wait' { Start-Sleep -Milliseconds ([int] $rest) }
        'clear' { [System.Windows.Forms.Clipboard]::Clear() }
        'clipboard' { $answers.Add("clipboard: `"$([System.Windows.Forms.Clipboard]::GetText())`"") }
        'cursor' { $answers.Add("cursor: $([Pamba.Hand]::Cursor())") }
        'touch' {
            $from = $rest -split '\s+' | ForEach-Object { [int] $_ }
            if (-not $script:fingers) {
                if (-not [Pamba.Hand]::InitializeTouchInjection(1, 1)) { throw 'Windows refused to let a finger be injected' }
                $script:fingers = $true
            }
            if (-not [Pamba.Hand]::Finger($from[0], $from[1], 0x10006)) { throw "the finger did not go down at $($from[0]) $($from[1])" }
            Start-Sleep -Milliseconds 60
            foreach ($part in 1..20) {
                $x = $from[0] + [int](($from[2] - $from[0]) * $part / 20)
                $y = $from[1] + [int](($from[3] - $from[1]) * $part / 20)
                [void][Pamba.Hand]::Finger($x, $y, 0x20006)
                Start-Sleep -Milliseconds 30
            }
            [void][Pamba.Hand]::Finger($from[2], $from[3], 0x40000)
            Start-Sleep -Milliseconds 120
        }
        'focus' { $answers.Add((Get-Focus)) }
        default { throw "a step this script does not know: $step" }
    }
}

Set-Content -LiteralPath $Out -Value $answers -Encoding utf8
