const path = require('path');

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  devServer: {
    host: '0.0.0.0',
    port: 18080
  },
  mode: "development"
};
