// lib.rs — hluau library: HTML/CSS to Luau UI transpiler.
// Pipeline: HTML parse → CSS resolve → layout transform → Luau codegen.

pub mod dom;
pub mod parser;
pub mod style;
pub mod layout;
pub mod codegen;

use anyhow::Result;

/// Compiles HTML+CSS into a Luau module that returns the root ScreenGui.
pub fn compile(html: &str, css: &str) -> Result<String> {
    compile_internal(html, css, false)
}

pub fn compile_standalone(html: &str, css: &str) -> Result<String> {
    compile_internal(html, css, true)
}

fn compile_internal(html: &str, css: &str, standalone: bool) -> Result<String> {
    let mut dom = parser::html::parse(html)?
        .unwrap_or_else(|| dom::LuauNode::new("Frame", "root"));
    style::resolver::resolve(&mut dom, css)?;
    let laid_out = layout::engine::transform(dom);
    if standalone {
        codegen::luau::generate_standalone(&laid_out)
    } else {
        codegen::luau::generate(&laid_out)
    }
}
