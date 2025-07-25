# LCAP 组件库按需引入插件（SWC版）
[![Test](https://github.com/netease-lcap/swc-plugin-import-lcap/actions/workflows/test.yml/badge.svg)](https://github.com/netease-lcap/swc-plugin-import-lcap/actions/workflows/test.yml)

## 作用

处理低代码组件库的按需引入

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

## 效果

```js
// before
import { ElButton, ElInput, ElSelect } from '@lcap/element-plus';

// after
import { ElButton } from '@lcap/element-plus/es/button';
import { ElInput } from '@lcap/element-plus/es/input';
import { ElSelect } from '@lcap/element-plus/es/select';
```