#![allow(clippy::not_unsafe_ptr_arg_deref)]
use swc_core::{
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod transform;
use transform::{modularize_imports, Config};

#[plugin_transform]
fn transform_imports_plugin(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let packages = serde_json::from_str(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config for @lcap/swc-plugin-import"),
    )
    .expect("invalid modules");

    program.apply(modularize_imports(&Config { packages }))
}
