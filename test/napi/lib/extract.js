const assert = require("assert");

const addon = require("..");

describe("Extractors", () => {
  it("Single Argument", () => {
    assert.strictEqual(addon.extract_single_add_one(41), 42);
  });

  it("Kitchen Sink", () => {
    const symbol = Symbol("Test");
    const values = [
      true,
      42,
      undefined,
      "hello",
      new Date(),
      symbol,
      new ArrayBuffer(100),
      new Uint8Array(Buffer.from("Buffer")),
      Buffer.from("Uint8Array"),
    ];

    // Pass `null` and `undefined` for `None`
    assert.deepStrictEqual(addon.extract_values(...values, null), [
      ...values,
      undefined,
      undefined,
    ]);

    // Pass values for optional
    assert.deepStrictEqual(addon.extract_values(...values, 100, "exists"), [
      ...values,
      100,
      "exists",
    ]);
  });

  it("Buffers", () => {
    const test = (TypedArray) => {
      const buf = new ArrayBuffer(24);
      const view = new TypedArray(buf);

      view[0] = 8;
      view[1] = 16;
      view[2] = 18;

      assert.strictEqual(addon.extract_buffer_sum(view), 42);
    };

    test(Uint8Array);
    test(Uint16Array);
    test(Uint32Array);
    test(Int8Array);
    test(Int16Array);
    test(Int32Array);
    test(Float32Array);
    test(Float64Array);
  });

  it("TypedArray", () => {
    assert.deepStrictEqual(
      Buffer.from(addon.bufferConcat(Buffer.from("abc"), Buffer.from("def"))),
      Buffer.from("abcdef")
    );

    assert.deepStrictEqual(
      Buffer.from(addon.stringToBuf("Hello, World!")),
      Buffer.from("Hello, World!")
    );
  });

  it("JSON", () => {
    assert.strictEqual(addon.extract_json_sum([1, 2, 3, 4]), 10);
    assert.strictEqual(addon.extract_json_sum([8, 16, 18]), 42);
  });

  it("Either", () => {
    assert.strictEqual(addon.extractEither("hello"), "String: hello");
    assert.strictEqual(addon.extractEither(42), "Number: 42");

    assert.throws(
      () => addon.extractEither({}),
      (err) => {
        assert.match(err.message, /expected either.*String.*f64/);
        assert.match(err.left.message, /expected string/);
        assert.match(err.right.message, /expected number/);

        return true;
      }
    );
  });

  it("With", async () => {
    assert.strictEqual(await addon.sleepWithJs(1.5), 1.5);
    assert.strictEqual(await addon.sleepWithJsSync(1.5), 1.5);
    assert.strictEqual(await addon.sleepWith(1.5), 1.5);
    assert.strictEqual(await addon.sleepWithSync(1.5), 1.5);
  });
});
