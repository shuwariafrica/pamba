<#
.SYNOPSIS
    Listens, as a screen reader does, to what one program's window says and to where the keyboard goes in it,
    and writes each, a line apiece, in the order Windows told them.

.DESCRIPTION
    Run on the signed-in desktop beside the program. It runs under Windows PowerShell 5.1, which a clean Windows 11
    has, since UI Automation's managed client is a .NET Framework assembly. It hears UI Automation's notification
    event from every element of the program's window, and the moves of the keyboard to an element of the program,
    from the moment it starts until its time is up, and then writes "done". A notification's line holds its words,
    its kind, its processing and its activity, so a program's words can be told from another's:

      notification: "The answer has arrived" kind Other processing All activity "pamba.announce"
      focus: "The field" ControlType.Edit

    A line is written as soon as it is heard, so what was heard before a failure stays in the file.

.PARAMETER ProcessId
    The program's Windows pid.

.PARAMETER Out
    The file the lines are written to.

.PARAMETER Seconds
    How long it listens.
#>
[CmdletBinding()]
[OutputType([void])]
param(
    [Parameter(Mandatory)][ValidateRange(1, [int]::MaxValue)][int] $ProcessId,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string] $Out,
    [ValidateRange(1, 3600)][int] $Seconds = 90
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
# UI Automation calls its handlers on threads of its own, where a script block has no runspace to run in, so the
# handlers are a class's and hand each line over through a queue.
Add-Type -ReferencedAssemblies UIAutomationClient, UIAutomationTypes -TypeDefinition @'
using System;
using System.Collections.Concurrent;
using System.Windows.Automation;

public static class PambaEars {
    public static readonly BlockingCollection<string> Heard = new BlockingCollection<string>();

    public static void Listen(AutomationElement window, int process) {
        Automation.AddAutomationEventHandler(AutomationElement.NotificationEvent, window, TreeScope.Subtree,
            (sender, args) => {
                NotificationEventArgs said = (NotificationEventArgs)args;
                Heard.Add(String.Format("notification: \"{0}\" kind {1} processing {2} activity \"{3}\"",
                    said.DisplayString, said.NotificationKind, said.NotificationProcessing, said.ActivityId));
            });
        Automation.AddAutomationFocusChangedEventHandler((sender, args) => {
            AutomationElement element = sender as AutomationElement;
            if (element == null) { return; }
            try {
                AutomationElement.AutomationElementInformation now = element.Current;
                if (now.ProcessId != process) { return; }
                Heard.Add(String.Format("focus: \"{0}\" {1}", now.Name, now.ControlType.ProgrammaticName));
            } catch (ElementNotAvailableException) {
                // The element went before it could be read: the next move is heard all the same.
            }
        });
    }
}
'@

$owned = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
$window = [System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $owned)
if ($null -eq $window) { throw "process $ProcessId has no window to listen to" }

Set-Content -LiteralPath $Out -Value @() -Encoding utf8
[PambaEars]::Listen($window, $ProcessId)
$until = [DateTime]::UtcNow.AddSeconds($Seconds)
$line = $null
while ($true) {
    $left = [int]($until - [DateTime]::UtcNow).TotalMilliseconds
    if ($left -le 0) { break }
    if ([PambaEars]::Heard.TryTake([ref] $line, $left)) {
        Add-Content -LiteralPath $Out -Value $line -Encoding utf8
    }
}
[System.Windows.Automation.Automation]::RemoveAllEventHandlers()
Add-Content -LiteralPath $Out -Value 'done' -Encoding utf8
