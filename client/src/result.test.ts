import { Result } from "./result";

describe('andThen', () => {
  test('should trigger op on success', async () => {
    let v = await Result.Ok("Fish")
        .andThen(async value => Result.Ok(value))

    expect(v.unwrap()).toBe("Fish");
  });

  test('should not trigger op on failure', async () => {
    let x = false;
    let v = await Result.Err("error").andThen(async value => {
      x = true;
      return Result.Ok(value)
    });

    expect(x).toBe(false);
    expect(v.error).toBe("error");
  })
});