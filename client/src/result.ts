export class Result<T> {
  readonly result;
  readonly error;

  private constructor(result: T | undefined, error: string | undefined) {
    this.result = result;
    this.error = error;
  }

  static Ok<T>(result: T) {
    return new Result<T>(result, undefined);
  }

  static Err<T>(error: string) {
    return new Result<T>(undefined, error);
  }

  async andThen<U>(op: (value: T) => Promise<Result<U>>) {
    return this.match({
      ok: (value) => op(value),
      err: (err) => Promise.resolve(Result.Err(err))
    })
  }

  async orElse(op: (error: string) => Promise<Result<T> | void>) {
    return this.match({
      ok: (value) => Promise.resolve(Result.Ok(value)),
      err: (msg) => op(msg)
    })
  }

  // andThen<U>(op: (value: T) => Result<U>) {
  //   return this.match({
  //     ok: (value) => op(value),
  //     err: (err) => Result.Err(err)
  //   })
  // }

  isOk() {
    return this.result !== undefined;
  }

  isErr() {
    return this.error !== undefined;
  }

  unwrap() {
    if (this.isErr()) {
      throw new Error("Unhandled error: " + this.error);
    }
    return this.result as T;
  }

  ifOk(callback: (value: T) => void) {
    if (this.isOk()) {
      callback(this.unwrap());
      return true;
    }
    return false;
  }

  match<R>(handle: { ok: (value: T) => R, err: (err: string) => R } ): R {
    if (this.isOk()) {
      return handle.ok(this.unwrap());
    } else {
      return handle.err(this.error as string);
    }
  }
}