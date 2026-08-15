@{
  # Invoke-ScriptAnalyzer -Path . -Recurse -Settings ./PSScriptAnalyzerSettings.psd1

  IncludeDefaultRules = $true

  # Information is off: it reports only positional parameters on release.ps1's Invoke-Git
  # wrapper, which forwards $args to git and so cannot name them.
  Severity = @('Error', 'Warning')

  ExcludeRules = @(
    # release.ps1 narrates to the operator running it, which is what Write-Host is for.
    'PSAvoidUsingWriteHost'
  )

  Rules = @{
    PSPlaceOpenBrace = @{
      Enable = $true
      OnSameLine = $true
      NewLineAfter = $true
      IgnoreOneLineBlock = $true
    }

    PSPlaceCloseBrace = @{
      Enable = $true
      NewLineAfter = $true
      IgnoreOneLineBlock = $true
      NoEmptyLineBefore = $true
    }

    PSUseConsistentIndentation = @{
      Enable = $true
      IndentationSize = 2
      PipelineIndentation = 'IncreaseIndentationForFirstPipeline'
      Kind = 'space'
    }

    PSUseConsistentWhitespace = @{
      Enable = $true
      CheckOpenBrace = $true
      CheckOpenParen = $true
      CheckOperator = $true
      CheckSeparator = $true
    }
  }
}
