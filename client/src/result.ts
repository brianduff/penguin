export class Result<T> {
  private readonly result;
  private readonly error;

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

  isOk() {
    return this.result !== undefined;
  }

  isErr() {
    return this.error === undefined;
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

  match(handle: { ok: (value: T) => void, err: (err: string) => void } ) {
    if (this.isOk()) {
      handle.ok(this.unwrap());
    } else {
      handle.err(this.error as string);
    }
  }
}