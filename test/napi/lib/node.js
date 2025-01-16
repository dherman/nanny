var addon = require("..");
var assert = require("chai").assert;

describe("Node Standard Library", function () {
  it("should print to console", function () {
    addon.call_console_log_and_error();
  });

  it("process.versions.node", function () {
    assert.strictEqual(addon.get_node_version(), process.versions.node);
  });
});
