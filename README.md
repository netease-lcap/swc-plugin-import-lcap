# SWC插件

## 作用

处理低代码组件库的按需引入

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