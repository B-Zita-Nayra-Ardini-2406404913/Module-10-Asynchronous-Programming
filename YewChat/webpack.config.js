const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const distPath = path.resolve(__dirname, 'dist');

module.exports = {
    mode: 'development',
    entry: './bootstrap.js',
    devServer: {
        port: 8000,
        historyApiFallback: true,
    },
    output: {
        path: distPath,
        filename: 'yewchat.js',
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [{ from: './static', to: distPath }],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '.'),
            outName: 'yewchat',
            outDir: path.resolve(__dirname, 'pkg'),
        }),
    ],
    experiments: {
        asyncWebAssembly: false,
        syncWebAssembly: true,
    },
};