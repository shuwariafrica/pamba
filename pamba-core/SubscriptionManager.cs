// Copyright (c) 2026 Ali Rashid. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Diagnostics;
using System.Threading.Tasks;

namespace Pamba;

/// <summary>
/// Manages subscription lifecycle via key-based diffing.
/// Internal to the runtime - not part of the public API.
/// </summary>
internal sealed class SubscriptionManager<TSub, TMsg> : IAsyncDisposable
    where TSub : IEquatable<TSub>, ISubscription<TMsg>
{
  private readonly Func<TSub, Dispatch<TMsg>, IAsyncDisposable> _starter;
  private readonly Action<PambaError> _onError;
  private readonly Dictionary<SubscriptionKey, (TSub Subscription, IAsyncDisposable Handle)> _active;
  private readonly HashSet<SubscriptionKey> _currentKeys;
  private readonly List<SubscriptionKey> _removalBuffer;

  internal SubscriptionManager(Func<TSub, Dispatch<TMsg>, IAsyncDisposable> starter, Action<PambaError> onError)
  {
    _starter = starter;
    _onError = onError;
    _active = [];
    _currentKeys = [];
    _removalBuffer = [];
  }

  /// <summary>
  /// Diff previous and current subscription sets.
  /// Same key + equal data = keep running. Same key + changed data = stop old, start new.
  /// New key = start. Removed key = cancel.
  /// Duplicate keys: first occurrence wins; subsequent duplicates are routed via OnRuntimeError and skipped.
  /// Removed/restarted handles are disposed via fire-and-forget (async cleanup runs in background).
  /// </summary>
  internal void Diff(ImmutableArray<TSub> current, Dispatch<TMsg> dispatch)
  {
    _currentKeys.Clear();

    foreach (TSub sub in current)
    {
      // Defence-in-depth: guard against default(SubscriptionKey) with null Value
      if (sub.Key.Value is null)
      {
        Trace.TraceError("Subscription with null key skipped. Use SubscriptionKey.From() to construct keys.");
        continue;
      }

      if (!_currentKeys.Add(sub.Key))
      {
        _onError(new PambaError.DuplicateSubscriptionKey(sub.Key));
        continue; // First-wins: skip duplicate
      }

      if (_active.TryGetValue(sub.Key, out var existing))
      {
        if (!existing.Subscription.Equals(sub))
        {
          DisposeHandleFireAndForget(existing.Handle);
          _active[sub.Key] = (sub, _starter(sub, dispatch));
        }
      }
      else
      {
        _active[sub.Key] = (sub, _starter(sub, dispatch));
      }
    }

    _removalBuffer.Clear();

    foreach (SubscriptionKey key in _active.Keys)
    {
      if (!_currentKeys.Contains(key))
      {
        _removalBuffer.Add(key);
      }
    }

    foreach (SubscriptionKey key in _removalBuffer)
    {
      DisposeHandleFireAndForget(_active[key].Handle);
      _active.Remove(key);
    }
  }

  /// <inheritdoc />
  public async ValueTask DisposeAsync()
  {
    foreach (var (_, handle) in _active.Values)
    {
      await DisposeHandleAsync(handle).ConfigureAwait(false);
    }

    _active.Clear();
  }

  // Fire-and-forget: used during Diff where awaiting is not possible (sync context).
  // Subscriptions with sync-only cleanup complete immediately. Subscriptions with
  // async cleanup (network connections etc.) run their cleanup in the background.
#pragma warning disable CA1031 // Subscription dispose must not block cleanup of remaining subscriptions
#pragma warning disable CA2012 // fire-and-forget: ValueTask used intentionally for background async cleanup
  private static void DisposeHandleFireAndForget(IAsyncDisposable handle) =>
      _ = DisposeHandleAsync(handle);
#pragma warning restore CA2012

  private static async ValueTask DisposeHandleAsync(IAsyncDisposable handle)
  {
    try
    {
      await handle.DisposeAsync().ConfigureAwait(false);
    }
    catch (Exception ex)
    {
      Trace.TraceError($"Subscription DisposeAsync threw an exception: {ex}");
    }
  }
#pragma warning restore CA1031
}
