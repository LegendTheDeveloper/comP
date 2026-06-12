// Unit tests for mergeExcludeIntoConfig (pure function, no VS Code API required)
import { strict as assert } from "assert";
import { mergeExcludeIntoConfig } from "../extension";

describe("mergeExcludeIntoConfig", () => {
  it("returns base object with exclude key when existing has none", () => {
    const result = mergeExcludeIntoConfig({}, ["env", "data"]);
    assert.deepStrictEqual(result["exclude"], ["env", "data"]);
  });

  it("preserves other keys in existing config", () => {
    const existing = { max_nodes: 100, additional_paths: ["/a"] };
    const result = mergeExcludeIntoConfig(existing, ["build"]);
    assert.strictEqual(result["max_nodes"], 100);
    assert.deepStrictEqual(result["additional_paths"], ["/a"]);
    assert.deepStrictEqual(result["exclude"], ["build"]);
  });

  it("overwrites existing exclude when values differ", () => {
    const existing = { exclude: ["old"] };
    const result = mergeExcludeIntoConfig(existing, ["new1", "new2"]);
    assert.deepStrictEqual(result["exclude"], ["new1", "new2"]);
  });

  it("returns unchanged base object when exclude is already equal", () => {
    const existing = { exclude: ["a", "b"] };
    const result = mergeExcludeIntoConfig(existing, ["a", "b"]);
    // Same values — exclude must equal the original
    assert.deepStrictEqual(result["exclude"], ["a", "b"]);
    // Other keys preserved (no extra mutation)
    assert.deepStrictEqual(Object.keys(result), ["exclude"]);
  });

  it("handles empty exclude array", () => {
    const result = mergeExcludeIntoConfig({ foo: 1 }, []);
    assert.deepStrictEqual(result["exclude"], []);
    assert.strictEqual(result["foo"], 1);
  });

  it("handles non-object existing (string) gracefully", () => {
    const result = mergeExcludeIntoConfig("invalid", ["env"]);
    assert.deepStrictEqual(result["exclude"], ["env"]);
  });

  it("handles null existing gracefully", () => {
    const result = mergeExcludeIntoConfig(null, ["env"]);
    assert.deepStrictEqual(result["exclude"], ["env"]);
  });

  it("handles array existing gracefully (treats as empty base)", () => {
    const result = mergeExcludeIntoConfig(["not", "an", "object"], ["env"]);
    assert.deepStrictEqual(result["exclude"], ["env"]);
  });
});
