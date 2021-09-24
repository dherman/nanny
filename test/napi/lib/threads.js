const addon = require('..');
const assert = require('chai').assert;

(function () {
  // These tests require GC exposed to shutdown properly; skip if it is not
  return typeof global.gc === 'function' ? describe : describe.skip;
})()('sync', function() {
  let uncaughtExceptionListeners = [];

  beforeEach(() => {
    uncaughtExceptionListeners = process.listeners("uncaughtException");
  });

  afterEach(() => {
    // Force garbage collection to shutdown `Channel`
    global.gc();

    // Restore listeners
    process.removeAllListeners("uncaughtException");
    uncaughtExceptionListeners.forEach(listener => process.on("uncaughtException", listener));
  });

  it('can create and deref a root', function () {
    const expected = {};
    const result = addon.useless_root(expected);

    assert.strictEqual(expected, result);
  });

  it('should be able to callback from another thread', function (cb) {
    addon.thread_callback(cb);
  });

  it('should be able to callback from multiple threads', function (cb) {
    const n = 4;
    const set = new Set([...new Array(n)].map((_, i) => i));

    addon.multi_threaded_callback(n, function (x) {
      if (!set.delete(x)) {
        cb(new Error(`Unexpected callback value: ${x}`));
      }

      if (set.size === 0) {
        cb();
      }
    });
  });

  it('should be able to use an async greeter', function (cb) {
    const greeter = addon.greeter_new('Hello, World!', function (greeting) {
      if (greeting === 'Hello, World!') {
        cb();
      } else {
        new Error('Greeting did not match');
      }
    });

    addon.greeter_greet(greeter);
  });

  it('should run callback on drop', function (cb) {
    // IIFE to allow GC
    (function () {
      addon.greeter_new('Hello, World!', function () {}, function () {
        // No assert needed; test will timeout
        cb();
      })
    })();

    global.gc();
  });

  it('should be able to unref channel', function () {
    // If the Channel is not unreferenced, the test runner will not cleanly exit
    addon.leak_channel();
  });

  it('should drop leaked Root from the global queue', function (cb) {
    addon.drop_global_queue(cb);

    // Asynchronously GC to give the task queue a chance to execute
    setTimeout(() => global.gc(), 10);
  });

  it('should handle panic in channel callback', function (cb) {
    process.removeAllListeners("uncaughtException");
    process.once("uncaughtException", (err) => {
      try {
        assert.instanceOf(err, Error);
        assert.ok(/channel/i.test(err));
        cb();
      } catch (err) {
        cb(err);
      }
    });

    try {
      addon.panic_in_channel();
    } catch (err) {
      cb(err);
    }
  });
});
