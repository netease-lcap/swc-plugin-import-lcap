use std::{borrow::Cow, collections::HashMap, sync::Arc};

use convert_case::{Case, Casing};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_cached::regex::CachedRegex;
use swc_common::util::take::Take;
use swc_ecma_ast::{ImportDecl, ImportSpecifier, ModuleExportName, *};
use swc_ecma_visit::{noop_visit_mut_type, visit_mut_pass, VisitMut, VisitMutWith};

static DUP_SLASH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"//").unwrap());

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Config {
    pub packages: HashMap<String, Arc<PackageConfig>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleConfig {
    pub src: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageConfig {
    pub es_dir: String,
    pub modules: HashMap<String, ModuleConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Transform {
    String(String),
    Vec(Vec<(String, String)>),
}

impl From<&str> for Transform {
    fn from(s: &str) -> Self {
        Transform::String(s.to_string())
    }
}
impl From<Vec<(String, String)>> for Transform {
    fn from(v: Vec<(String, String)>) -> Self {
        Transform::Vec(v)
    }
}

struct TransformImports<'a> {
    packages: Vec<(CachedRegex, &'a PackageConfig)>,
}

struct Rewriter<'a> {
    key: &'a str,
    config: &'a PackageConfig,
    group: Vec<&'a str>,
}

impl Rewriter<'_> {
    fn get_config_from_exports(&self, name_str: &str) -> Option<&ModuleConfig> {
        let modules = &self.config.modules;
        // 从exports中获取member的配置
        modules.get(name_str)
    }

    fn new_path(&self, name_str: Option<&str>) -> Atom {
        let package = self.key;
        let member_config = self.get_config_from_exports(name_str.unwrap_or_default());

        let new_path = match member_config {
            Some(ModuleConfig { src, is_default }) => {
                // package/{{src}}
                format!("{}/{}/{}", package, self.config.es_dir, src)
            }
            None => format!(
                "{}/{}/{}",
                package,
                self.config.es_dir,
                name_str.unwrap_or_default()
            ),
        };

        // new_path可能包含双斜杠，需要替换为单斜杠
        let new_path = DUP_SLASH_REGEX.replace_all(&new_path, "/");
        new_path.into()
    }

    fn rewrite_import(&self, old_decl: &ImportDecl) -> Vec<ImportDecl> {
        if old_decl.type_only || old_decl.with.is_some() {
            return vec![old_decl.clone()];
        }

        let mut out: Vec<ImportDecl> = Vec::with_capacity(old_decl.specifiers.len());

        for spec in &old_decl.specifiers {
            match spec {
                ImportSpecifier::Named(named_spec) => {
                    let name_str = named_spec
                        .imported
                        .as_ref()
                        .map(|x| match x {
                            ModuleExportName::Ident(x) => x.as_ref(),
                            ModuleExportName::Str(x) => x.value.as_ref(),
                        })
                        .unwrap_or_else(|| named_spec.local.as_ref());

                    let member_config = self.get_config_from_exports(name_str);
                    let mut is_default = true;

                    if let Some(member_config) = member_config {
                        is_default = member_config.is_default;
                    }

                    let new_path = self.new_path(Some(name_str));
                    let specifier = if is_default {
                        ImportSpecifier::Default(ImportDefaultSpecifier {
                            local: named_spec.local.clone(),
                            span: named_spec.span,
                        })
                    } else {
                        ImportSpecifier::Named(named_spec.clone())
                    };

                    out.push(ImportDecl {
                        specifiers: vec![specifier],
                        src: Box::new(Str::from(new_path.as_ref())),
                        span: old_decl.span,
                        type_only: false,
                        with: None,
                        phase: Default::default(),
                    });
                }
                _ => {
                    return vec![old_decl.clone()];
                }
            }
        }
        out
    }
}

impl TransformImports<'_> {
    fn should_rewrite<'a>(&'a self, name: &'a str) -> Option<Rewriter<'a>> {
        for (regex, config) in &self.packages {
            let group = regex.captures(name);
            if let Some(group) = group {
                let group = group
                    .iter()
                    .map(|x| x.map(|x| x.as_str()).unwrap_or_default())
                    .collect::<Vec<&str>>();

                return Some(Rewriter {
                    key: name,
                    config,
                    group,
                });
            }
        }
        None
    }
}

impl VisitMut for TransformImports<'_> {
    noop_visit_mut_type!();

    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        call.visit_mut_children_with(self);
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        let mut new_items: Vec<ModuleItem> = Vec::with_capacity(module.body.len());
        for item in module.body.take() {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
                    if decl.specifiers.is_empty() {
                        if let Some(rewriter) = self.should_rewrite(&decl.src.value) {
                            let new_path = rewriter.new_path(None);
                            let raw_with_quotes = Atom::from(format!("'{}'", new_path.as_ref()));
                            let new_src = Box::new(Str {
                                span: decl.src.span,
                                value: new_path.clone(),
                                raw: Some(raw_with_quotes),
                            });
                            let new_decl = ImportDecl {
                                src: new_src,
                                specifiers: vec![],
                                ..decl
                            };

                            new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(new_decl)));
                        } else {
                            new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(decl)));
                        }
                        continue;
                    }

                    match self.should_rewrite(&decl.src.value) {
                        Some(rewriter) => {
                            let rewritten = rewriter.rewrite_import(&decl);
                            new_items.extend(
                                rewritten
                                    .into_iter()
                                    .map(ModuleDecl::Import)
                                    .map(ModuleItem::ModuleDecl),
                            );
                        }
                        None => new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(decl))),
                    }
                }
                _ => {
                    new_items.push(item);
                }
            }
        }
        module.body = new_items;
    }
}

pub fn modularize_imports(config: &Config) -> impl '_ + Pass {
    let mut folder = TransformImports { packages: vec![] };

    for (k, v) in &config.packages {
        let mut k = Cow::Borrowed(k);
        if !k.starts_with('^') && !k.ends_with('$') {
            k = Cow::Owned(format!("^{k}$"));
        }
        folder.packages.push((
            CachedRegex::new(&k).expect("transform-imports: invalid regex"),
            v,
        ));
    }
    visit_mut_pass(folder)
}
