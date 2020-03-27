/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
const path = require('path');

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
    },
    devServer: {
        host: '0.0.0.0',
        port: 8080
    },
    mode: "development"
};
