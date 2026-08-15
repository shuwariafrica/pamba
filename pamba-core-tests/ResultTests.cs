// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using Xunit;

namespace Pamba.Tests;

public sealed class ResultTests
{
  [Fact]
  public void Ok_carries_its_value_and_reports_the_success_case()
  {
    Result<int, string> result = Result.Ok<int, string>(7);

    Assert.True(result.IsOk);
    Assert.False(result.IsErr);
    Assert.Equal(7, Assert.IsType<Result<int, string>.Ok>(result).Value);
  }

  [Fact]
  public void Err_carries_its_error_and_reports_the_failure_case()
  {
    Result<int, string> result = Result.Err<int, string>("no");

    Assert.True(result.IsErr);
    Assert.False(result.IsOk);
    Assert.Equal("no", Assert.IsType<Result<int, string>.Err>(result).Error);
  }

  [Fact]
  public void Map_transforms_the_success_value_and_leaves_a_failure_untouched()
  {
    Assert.Equal(14, Result.Ok<int, string>(7).Map(v => v * 2).DefaultValue(0));

    Result<int, string> mapped = Result.Err<int, string>("no").Map(v => v * 2);
    Assert.Equal("no", Assert.IsType<Result<int, string>.Err>(mapped).Error);
  }

  [Fact]
  public void MapErr_transforms_the_error_and_leaves_a_success_untouched()
  {
    Result<int, int> mapped = Result.Err<int, string>("failure").MapErr(e => e.Length);
    Assert.Equal(7, Assert.IsType<Result<int, int>.Err>(mapped).Error);

    Assert.Equal(7, Result.Ok<int, string>(7).MapErr(e => e.Length).DefaultValue(0));
  }

  [Fact]
  public void Bind_chains_on_success_and_short_circuits_on_failure()
  {
    Result<string, string> chained =
        Result.Ok<int, string>(7).Bind(v => Result.Ok<string, string>($"n={v}"));
    Assert.Equal("n=7", chained.DefaultValue(string.Empty));

    Result<string, string> shortCircuited =
        Result.Err<int, string>("no").Bind(v => Result.Ok<string, string>($"n={v}"));
    Assert.Equal("no", Assert.IsType<Result<string, string>.Err>(shortCircuited).Error);
  }

  [Fact]
  public void Bind_propagates_a_failure_produced_by_the_chained_operation()
  {
    Result<string, string> chained =
        Result.Ok<int, string>(7).Bind(_ => Result.Err<string, string>("inner"));

    Assert.Equal("inner", Assert.IsType<Result<string, string>.Err>(chained).Error);
  }

  [Fact]
  public void DefaultValue_yields_the_success_value_or_the_fallback()
  {
    Assert.Equal(7, Result.Ok<int, string>(7).DefaultValue(-1));
    Assert.Equal(-1, Result.Err<int, string>("no").DefaultValue(-1));
  }

  [Fact]
  public void DefaultWith_computes_the_fallback_from_the_error_only_on_failure()
  {
    Assert.Equal(7, Result.Ok<int, string>(7).DefaultWith(e => e.Length));
    Assert.Equal(7, Result.Err<int, string>("failure").DefaultWith(e => e.Length));
  }

  [Fact]
  public void Switch_invokes_exactly_the_branch_matching_the_case()
  {
    int okCalls = 0;
    int errCalls = 0;

    Result.Ok<int, string>(7).Switch(_ => okCalls++, _ => errCalls++);
    Assert.Equal(1, okCalls);
    Assert.Equal(0, errCalls);

    Result.Err<int, string>("no").Switch(_ => okCalls++, _ => errCalls++);
    Assert.Equal(1, okCalls);
    Assert.Equal(1, errCalls);
  }

  [Fact]
  public void Match_maps_both_cases_to_a_single_type()
  {
    Assert.Equal("ok:7", Result.Ok<int, string>(7).Match(v => $"ok:{v}", e => $"err:{e}"));
    Assert.Equal("err:no", Result.Err<int, string>("no").Match(v => $"ok:{v}", e => $"err:{e}"));
  }

  [Fact]
  public void Map_round_trips_through_an_inverse_transformation()
  {
    Result<int, string> original = Result.Ok<int, string>(7);

    int recovered = original.Map(v => v.ToString(System.Globalization.CultureInfo.InvariantCulture))
        .Map(s => int.Parse(s, System.Globalization.CultureInfo.InvariantCulture))
        .DefaultValue(-1);

    Assert.Equal(7, recovered);
  }

  [Fact]
  public void Combinators_reject_a_null_function()
  {
    Result<int, string> result = Result.Ok<int, string>(7);
    Func<int, int> nullMap = null!;
    Func<string, string> nullMapErr = null!;
    Func<int, Result<int, string>> nullBind = null!;
    Func<string, int> nullFallback = null!;
    Func<int, string> nullOnOk = null!;

    Assert.Throws<ArgumentNullException>(() => result.Map(nullMap));
    Assert.Throws<ArgumentNullException>(() => result.MapErr(nullMapErr));
    Assert.Throws<ArgumentNullException>(() => result.Bind(nullBind));
    Assert.Throws<ArgumentNullException>(() => result.DefaultWith(nullFallback));
    Assert.Throws<ArgumentNullException>(() => result.Switch(null!, _ => { }));
    Assert.Throws<ArgumentNullException>(() => result.Match(nullOnOk, _ => string.Empty));
  }
}
