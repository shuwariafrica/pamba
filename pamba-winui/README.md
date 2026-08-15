# Pamba.WinUI

WinUI 3 integration for the Pamba MVU runtime.

```shell
dotnet add package Pamba.WinUI
```

Requires .NET 10 / C# 14, `Pamba`, and `Microsoft.WindowsAppSDK`. Minimum platform: Windows 11 22H2.

## Starting the runtime

`WinUIRuntime` builds and starts the MVU loop. The builder enforces the correct
construction order at compile time. Projection is optional - call `.Start()` directly
after `.WithSubscriptionStarter()` if you have no UI projection.

```csharp
var projection = new AppProjection(mainWindow);

MvuRuntime<AppState, Msg, Cmd, Sub> runtime = WinUIRuntime
    .Create(program, mainWindow.DispatcherQueue)
    .WithCommandExecutor(commandExecutor.Execute)
    .WithSubscriptionStarter(subscriptionStarter.Start)
    .WithProjection(projection)
    .Start();
```

## State projection

`Projection<TState>` maps state changes to UI updates. It ships in `Pamba`, not this package,
so a projection class can live in your core project. Subclass it and register segments in
the constructor. Each segment receives a selector that identifies
a slice of state, and an action that updates the UI for that slice. Only segments whose
selected value has changed are called on each transition.

```csharp
public sealed class AppProjection : Projection<AppState>
{
    public AppProjection(MainWindow window)
    {
        Segment(s => s.Auth, auth => ProjectAuth(window, auth));
        Segment(s => s.CurrentModule, mod => ProjectNavigation(window, mod));
        Segment(s => s.Items, items => ProjectItems(window, items));
    }
}
```

All segments run against the initial state once at startup via `ProjectInitial`.

The selector return type must implement `IEquatable<TSegment>`. C# records satisfy this automatically.

### Transition-aware segments

When a segment needs the previous value to determine transition behaviour (e.g. animation
direction), use the three-parameter overload:

```csharp
Segment(
    s => s.CurrentModule,
    mod => SetModuleWithoutAnimation(mod),        // initial projection
    (old, @new) => AnimateModuleSwitch(old, @new));  // transition projection
```

`projectInitial` fires once at startup. `projectTransition` fires on each state change
that alters the selected value, receiving both old and new values.

## Timer Subscriptions

Pre-built helpers for timer-based subscriptions. Use them inside your
`SubscriptionStarter` delegate. Pass through the `onError` callback from the runtime
so tick/event handler exceptions route as `PambaError.SubscriptionFaulted`:

```csharp
IAsyncDisposable StartSubscription(Sub subscription, Dispatch<Msg> dispatch, Action<Exception> onError) =>
    subscription switch
    {
      Sub.RefreshTimer t => TimerSubscription.Start(
          interval: t.Interval,
          createMessage: () => new Msg.RefreshTick(),
          dispatch: dispatch,
          onError: onError,
          dispatcherQueue: _dispatcherQueue),

      Sub.SearchDebounce d => DelayedSubscription.Start(
          delay: d.Delay,
          createMessage: () => new Msg.DebounceComplete(),
          dispatch: dispatch,
          onError: onError,
          dispatcherQueue: _dispatcherQueue),

      _ => throw new InvalidOperationException($"Unknown subscription: {subscription}")
    };
```

Both return `IAsyncDisposable`. The runtime manages their lifecycle via subscription
diffing - you do not need to dispose them manually.

## Property Changed Subscription

Bridges `INotifyPropertyChanged` events into the MVU loop. Use this to subscribe to
external observable state such as Lugha's `LocaleHost` or system theme providers.

```csharp
Sub.LocaleChanged s => PropertyChangedSubscription.Start(
    source: localeHost,
    propertyName: "Current",
    createMessage: () => new Msg.LocaleChanged(localeHost.Current.Culture.Name),
    dispatch: dispatch,
    onError: onError,
    dispatcherQueue: _dispatcherQueue),
```

The `dispatcherQueue` parameter ensures `createMessage` runs on the UI thread regardless
of which thread raised the property change event.

## Command Debouncer

Wraps a `CommandExecutor` to debounce high-frequency commands. Each invocation
cancels the previous pending execution. `Execute` returns `CommandResult<TMsg>.Ok`
immediately (actual execution is deferred), making it directly usable as a
`CommandExecutor<TCmd, TMsg>`:

```csharp
var debounced = new CommandDebouncer<Cmd, Msg>(
    delay: TimeSpan.FromMilliseconds(300),
    inner: actualExecutor,
    dispatcherQueue: dispatcherQueue);

// debounced.Execute is a CommandExecutor<Cmd, Msg>
```

### Shutdown

`FlushAsync` executes pending work, then prevents further scheduling.
`DiscardPending` cancels pending work without executing it; the debouncer remains
usable (e.g. session expiry where the pending state should not be persisted).

```csharp
// Graceful shutdown: execute pending, then dispose
await debounced.FlushAsync();
await debounced.DisposeAsync();

// Session expiry: discard pending, keep debouncer alive
debounced.DiscardPending();
```

### Multiple Debounce Domains

Use separate `CommandDebouncer` instances per domain and route in the command executor:

```csharp
var searchDebounce = new CommandDebouncer<Cmd, Msg>(
    TimeSpan.FromMilliseconds(200), innerExecutor, dispatcherQueue);
var prefsDebounce = new CommandDebouncer<Cmd, Msg>(
    TimeSpan.FromMilliseconds(500), innerExecutor, dispatcherQueue);

ValueTask<CommandResult<Msg>> Execute(Cmd cmd, Dispatch<Msg> d, CancellationToken ct) =>
    cmd switch
    {
        Cmd.Search    => searchDebounce.Execute(cmd, d, ct),
        Cmd.SavePrefs => prefsDebounce.Execute(cmd, d, ct),
        _             => innerExecutor(cmd, d, ct),
    };
```

## Localisation with Lugha

[Lugha](https://github.com/arashi01/lugha) is a typed localisation library for .NET 10 with
compile-time-enforced text contracts and CLDR pluralisation. The active locale lives in MVU state;
a projection segment keeps the Lugha `LocaleHost` in sync so that XAML text bindings update
automatically on locale switch.

Add the locale to your state type:

```csharp
public sealed record AppState
{
    public required IAppLocale Locale { get; init; }
    // ...
}
```

Handle locale switching in `Update`. `Resolve` is total - returns the registry's default
locale when no match is found:

```csharp
Msg.LocaleSwitched m =>
    (state with { Locale = registry.Resolve(m.LanguageTag) }, []),
```

Register a segment in your projection that calls `SetLocale` when the locale changes:

```csharp
public sealed class AppProjection : Projection<AppState>
{
    public AppProjection(MainWindow window, WinUILocaleHost<IAppLocale> localeHost)
    {
        Segment(s => s.Locale, locale => localeHost.SetLocale(locale));
        Segment(s => s.Auth, auth => ProjectAuth(window, auth));
        // ...
    }
}
```

For runtime locale changes originating externally (e.g. system language change), use
`PropertyChangedSubscription` to bridge `LocaleHost.PropertyChanged` into the MVU loop.

Wire everything at startup:

```csharp
LocaleRegistry<IAppLocale> registry = LocaleRegistry<IAppLocale>
    .Create(new EnGbLocale(), new ArSaLocale())
    .Match(ok => ok, err => throw new InvalidOperationException($"Duplicate: {err.Tag}"));

WinUILocaleHost<IAppLocale> localeHost =
    LocaleHostFactory.Create(new EnGbLocale(), mainWindow.DispatcherQueue);

var projection = new AppProjection(mainWindow, localeHost);

_runtime = WinUIRuntime
    .Create(program, mainWindow.DispatcherQueue)
    .WithCommandExecutor(executor)
    .WithSubscriptionStarter(starter)
    .WithProjection(projection)
    .Start();
```

Bind text and RTL flow direction in XAML:

```xml
<Grid FlowDirection="{x:Bind localeHost.FlowDirection, Mode=OneWay}">
  <TextBlock Text="{x:Bind localeHost.Current.Navigation.Dashboard, Mode=OneWay}" />
  <TextBlock Text="{x:Bind localeHost.Current.Connection.Connected(ViewModel.Host), Mode=OneWay}" />
</Grid>
```

For system language synchronisation (persistent `PrimaryLanguageOverride`), call
`SystemLanguageSync.TryApply` inside the locale segment action. See the
[Lugha.WinUI documentation](https://github.com/arashi01/lugha) for details.

## Related Packages

- **Pamba** - Framework-agnostic core: contracts, dispatch loop, command/subscription infrastructure.
- **Pamba.Testing** - `Scenario` for multi-step flow testing using `program.Step()` / `program.Initialize()`.

## Licence

Apache License 2.0
