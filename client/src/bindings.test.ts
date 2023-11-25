import { RuleKind } from "./bindings/Rule"

// This test probably doesn't need to exist, but it's helpful
// for me to confirm my understanding of how TypeScript converts back and
// forth from strings to enums.

describe('bindings', () => {
  test('funny test', () => {
    const obj = {
      allow: RuleKind.ALLOW_HTTP_ACCESS,
      deny: RuleKind.DENY_HTTP_ACCESS
    };

    const json = JSON.stringify(obj);
    expect(json).toBe('{"allow":"allow_http_access","deny":"deny_http_access"}')

    const value = {
      "allow": "allow_http_access",
      "deny": "deny_http_access"
    };

    interface Foo {
      allow: RuleKind,
      deny: RuleKind
    };
    let converted = value as Foo;

    expect(converted.allow).toBe(RuleKind.ALLOW_HTTP_ACCESS);
    expect(converted.deny).toBe(RuleKind.DENY_HTTP_ACCESS);

  })
})