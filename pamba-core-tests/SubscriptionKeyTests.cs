// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using Xunit;

namespace Pamba.Tests;

public sealed class SubscriptionKeyTests
{
  [Fact]
  public void From_accepts_a_non_empty_value_and_preserves_it()
  {
    Assert.Equal("ticker", SubscriptionKey.From("ticker").Value);
  }

  [Fact]
  public void From_rejects_null_and_empty_as_a_construction_defect()
  {
    Assert.Throws<ArgumentNullException>(() => SubscriptionKey.From(null!));
    Assert.Throws<ArgumentException>(() => SubscriptionKey.From(string.Empty));
  }

  [Fact]
  public void TryFrom_returns_the_key_for_a_usable_value()
  {
    Result<SubscriptionKey, string> result = SubscriptionKey.TryFrom("ticker");

    Assert.True(result.IsOk);
    Assert.Equal("ticker", result.Match(k => k.Value, _ => string.Empty));
  }

  [Fact]
  public void TryFrom_returns_an_error_for_null_and_empty_rather_than_throwing()
  {
    Assert.True(SubscriptionKey.TryFrom(null).IsErr);
    Assert.True(SubscriptionKey.TryFrom(string.Empty).IsErr);
  }

  [Fact]
  public void Keys_with_the_same_value_are_equal_so_the_runtime_treats_them_as_one_subscription()
  {
    Assert.Equal(SubscriptionKey.From("ticker"), SubscriptionKey.From("ticker"));
    Assert.NotEqual(SubscriptionKey.From("ticker"), SubscriptionKey.From("poller"));
  }
}
