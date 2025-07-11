# SWC插件
[![Test](https://github.com/netease-lcap/swc-plugin-import-lcap/actions/workflows/test.yml/badge.svg)](https://github.com/netease-lcap/swc-plugin-import-lcap/actions/workflows/test.yml)

## 作用

处理低代码组件库的按需引入，当前只处理ImportSpecifier::Named，例如：import { A } from 'package' 会转换为 import A from 'package/{{es}}/{{file}}'

## 构建

```sh
npm pack
```

## 使用

```js
// rspack.config.js

      {
        test: /\.[mc]?js$/,
        type: 'javascript/auto',
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                parser: {
                  syntax: 'ecmascript',
                },
                experimental: {
                  plugins: [
                    ['@lcap/swc-plugin-import', {
                      '@lcap/element-plus': {
                        esDir: 'es',
                        modules: require('@lcap/element-plus/es/modules.json').exports,
                      }
                    }]
                  ]
                }
              },
            },
          }
        ],
      }

```
