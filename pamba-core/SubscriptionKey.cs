// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;

namespace Pamba;

/// <summary>
/// Strongly-typed key identifying a subscription instance.
/// Two subscriptions with the same <see cref="SubscriptionKey"/> are considered the same logical subscription.
/// Construct via <see cref="SubscriptionKeyExtensions"/>.
/// </summary>
public readonly record struct SubscriptionKey
{
  /// <summary>The string value of this subscription key.</summary>
  public string Value { get; }

  internal SubscriptionKey(string value) => Value = value;
}

/// <summary>
/// Factory members for <see cref="SubscriptionKey"/>.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
public static class SubscriptionKeyExtensions
{
  extension(SubscriptionKey)
  {
    /// <summary>
    /// Create a subscription key from a known-valid string.
    /// Throws <see cref="ArgumentException"/> for null or empty values (programming bug).
    /// </summary>
    /// <param name="value">Non-null, non-empty key string.</param>
    public static SubscriptionKey From(string value)
    {
      ArgumentException.ThrowIfNullOrEmpty(value);
      return new SubscriptionKey(value);
    }

    /// <summary>
    /// Create a subscription key from a runtime-provided string.
    /// Returns <see cref="Result{T, TErr}.Err"/> when <paramref name="value"/> is null or empty.
    /// </summary>
    public static Result<SubscriptionKey, string> TryFrom(string? value) =>
        string.IsNullOrEmpty(value)
            ? new Result<SubscriptionKey, string>.Err("Subscription key must not be null or empty.")
            : new Result<SubscriptionKey, string>.Ok(new SubscriptionKey(value));
  }
}
#pragma warning restore CA1034
