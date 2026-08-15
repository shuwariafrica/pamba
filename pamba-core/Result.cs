// Copyright (c) 2026 Shuwari Africa. Licensed under the Apache License, Version 2.0.
// See LICENSE in the project root for licence information.

using System;
using System.Diagnostics;

namespace Pamba;

/// <summary>
/// Either success (<see cref="Ok"/>) or failure (<see cref="Err"/>). Derivation is closed
/// to this assembly, so a match covering both cases stays total across future versions.
/// </summary>
/// <typeparam name="T">Success value type.</typeparam>
/// <typeparam name="TErr">Error value type.</typeparam>
public abstract record Result<T, TErr>
{
  private protected Result() { }

#pragma warning disable CA1034 // Sealed ADT hierarchy: nested public types form a closed discriminated union. private protected constructor prevents external derivation.
  /// <summary>The successful outcome, carrying <see cref="Value"/>.</summary>
  /// <param name="Value">The success value.</param>
  public sealed record Ok(T Value) : Result<T, TErr>;

  /// <summary>The failure outcome, carrying <see cref="Error"/>.</summary>
  /// <param name="Error">The error value.</param>
  public sealed record Err(TErr Error) : Result<T, TErr>;
#pragma warning restore CA1034
}

/// <summary>
/// Factory methods for <see cref="Result{T, TErr}"/>.
/// </summary>
public static class Result
{
  /// <summary>Create a successful result carrying <paramref name="value"/>.</summary>
  public static Result<T, TErr> Ok<T, TErr>(T value) => new Result<T, TErr>.Ok(value);

  /// <summary>Create a failure result carrying <paramref name="error"/>.</summary>
  public static Result<T, TErr> Err<T, TErr>(TErr error) => new Result<T, TErr>.Err(error);
}

/// <summary>
/// Extension members for <see cref="Result{T, TErr}"/>.
/// </summary>
#pragma warning disable CA1034 // C# 14 extension blocks appear as nested types to the analyser
public static class ResultExtensions
{
  extension<T, TErr>(Result<T, TErr> r)
  {
    /// <summary>True when the result is <see cref="Result{T, TErr}.Ok"/>.</summary>
    public bool IsOk => r is Result<T, TErr>.Ok;

    /// <summary>True when the result is <see cref="Result{T, TErr}.Err"/>.</summary>
    public bool IsErr => r is Result<T, TErr>.Err;

    /// <summary>
    /// Transform the success value. Returns <see cref="Result{T, TErr}.Err"/> unchanged.
    /// </summary>
    public Result<TResult, TErr> Map<TResult>(Func<T, TResult> f)
    {
      ArgumentNullException.ThrowIfNull(f);
      return r switch
      {
        Result<T, TErr>.Ok ok => new Result<TResult, TErr>.Ok(f(ok.Value)),
        Result<T, TErr>.Err err => new Result<TResult, TErr>.Err(err.Error),
        _ => throw new UnreachableException($"unhandled {r.GetType().Name}"),
      };
    }

    /// <summary>
    /// Transform the error value. Returns <see cref="Result{T, TErr}.Ok"/> unchanged.
    /// </summary>
    public Result<T, TNewErr> MapErr<TNewErr>(Func<TErr, TNewErr> f)
    {
      ArgumentNullException.ThrowIfNull(f);
      return r switch
      {
        Result<T, TErr>.Ok ok => new Result<T, TNewErr>.Ok(ok.Value),
        Result<T, TErr>.Err err => new Result<T, TNewErr>.Err(f(err.Error)),
        _ => throw new UnreachableException($"unhandled {r.GetType().Name}"),
      };
    }

    /// <summary>
    /// Chain a fallible operation on the success value.
    /// Returns <see cref="Result{T, TErr}.Err"/> unchanged.
    /// </summary>
    public Result<TNew, TErr> Bind<TNew>(Func<T, Result<TNew, TErr>> f)
    {
      ArgumentNullException.ThrowIfNull(f);
      return r switch
      {
        Result<T, TErr>.Ok ok => f(ok.Value),
        Result<T, TErr>.Err err => new Result<TNew, TErr>.Err(err.Error),
        _ => throw new UnreachableException($"unhandled {r.GetType().Name}"),
      };
    }

    /// <summary>
    /// Unwrap the success value, or return <paramref name="fallback"/> on error.
    /// </summary>
    public T DefaultValue(T fallback) =>
        r is Result<T, TErr>.Ok ok ? ok.Value : fallback;

    /// <summary>
    /// Unwrap the success value, or compute a fallback from the error.
    /// </summary>
    public T DefaultWith(Func<TErr, T> fallback)
    {
      ArgumentNullException.ThrowIfNull(fallback);
      return r switch
      {
        Result<T, TErr>.Ok ok => ok.Value,
        Result<T, TErr>.Err err => fallback(err.Error),
        _ => throw new UnreachableException($"unhandled {r.GetType().Name}"),
      };
    }

    /// <summary>
    /// Invoke <paramref name="onOk"/> for success or <paramref name="onErr"/> for failure.
    /// </summary>
    public void Switch(Action<T> onOk, Action<TErr> onErr)
    {
      ArgumentNullException.ThrowIfNull(onOk);
      ArgumentNullException.ThrowIfNull(onErr);
      if (r is Result<T, TErr>.Ok ok)
      {
        onOk(ok.Value);
      }
      else if (r is Result<T, TErr>.Err err)
      {
        onErr(err.Error);
      }
      else
      {
        throw new UnreachableException($"unhandled {r.GetType().Name}");
      }
    }

    /// <summary>
    /// Map both outcomes to a single value.
    /// </summary>
    public TOut Match<TOut>(Func<T, TOut> onOk, Func<TErr, TOut> onErr)
    {
      ArgumentNullException.ThrowIfNull(onOk);
      ArgumentNullException.ThrowIfNull(onErr);
      return r switch
      {
        Result<T, TErr>.Ok ok => onOk(ok.Value),
        Result<T, TErr>.Err err => onErr(err.Error),
        _ => throw new UnreachableException($"unhandled {r.GetType().Name}"),
      };
    }
  }
}
#pragma warning restore CA1034
