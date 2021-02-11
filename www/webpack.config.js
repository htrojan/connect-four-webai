const path = require('path')
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin")
const HTMLWebpackPlugin = require("html-webpack-plugin")
const CopyPlugin = require("copy-webpack-plugin")

    // experiments: {
    //     // asyncWebAssembly: true
    //     syncWebAssembly: true
    // },
    // rules: [
    //     {
    //         test: /\.wasm$/,
    //         type: "webassembly/sync",
    //     }
    // ],
module.exports = {
    entry: "./bootstrap.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bundle.js",
    },
    plugins: [
      new WasmPackPlugin({
          crateDirectory: path.resolve(__dirname, "../"),
          outName: "connect_four",
      }),
      new CopyPlugin({
          patterns: [
              {from: path.resolve(__dirname, "index.html")}
          ]
      })
    ],
    experiments: {
        // asyncWebAssembly: true
        syncWebAssembly: true
    },
    mode: "production",
    devServer: {
        contentBase: "./dist"
    }
}