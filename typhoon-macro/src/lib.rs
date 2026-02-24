use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Token, braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

struct NodeMethod {
    name: Ident,
    arg: Expr,
}

impl Parse for NodeMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![.]>()?;
        let name: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let arg: Expr = content.parse()?;
        Ok(NodeMethod { name, arg })
    }
}

/// Grammar: `tag [.method(arg)]* ['{' children '}']`
struct TpNode {
    tag: Ident,
    methods: Vec<NodeMethod>,
    children: Vec<TpChild>,
}

enum TpChild {
    Node(TpNode),
    Text(LitStr),
    Embed(Expr), // (expr) — embeds an Element returned by a component/function
}

impl Parse for TpNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag: Ident = input.parse()?;

        let mut methods = Vec::new();
        while input.peek(Token![.]) {
            methods.push(input.parse::<NodeMethod>()?);
        }

        let mut children = Vec::new();
        if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            while !content.is_empty() {
                if content.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in content);
                    children.push(TpChild::Embed(inner.parse()?));
                } else if content.peek(LitStr) {
                    children.push(TpChild::Text(content.parse()?));
                } else {
                    children.push(TpChild::Node(content.parse()?));
                }
            }
        }

        Ok(TpNode {
            tag,
            methods,
            children,
        })
    }
}

struct TpInput(TpNode);

impl Parse for TpInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TpInput(input.parse()?))
    }
}

fn generate_node(node: &TpNode) -> TokenStream2 {
    let tag = node.tag.to_string();

    let mut stmts = quote! {
        let __el = ::typhoon_core::create_element(#tag);
    };

    for method in &node.methods {
        let method_name = method.name.to_string();
        let arg = &method.arg;

        match method_name.as_str() {
            "text" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_text_content(&__el, &#arg);
                };
            }
            "class" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_class(&__el, #arg);
                };
            }
            "style" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_style(&__el, #arg);
                };
            }
            "onclick" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_onclick(&__el, #arg);
                };
            }
            "id" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_attribute(&__el, "id", #arg);
                };
            }
            "placeholder" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_attribute(&__el, "placeholder", #arg);
                };
            }
            "value" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_attribute(&__el, "value", &#arg);
                };
            }
            "oninput" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_oninput(&__el, #arg);
                };
            }
            "onkeydown" => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_onkeydown(&__el, #arg);
                };
            }
            _ => {
                // Generic attribute fallback
                let attr_name = method_name;
                stmts = quote! {
                    #stmts
                    ::typhoon_core::set_attribute(&__el, #attr_name, &#arg);
                };
            }
        }
    }

    for child in &node.children {
        match child {
            TpChild::Node(child_node) => {
                let child_code = generate_node(child_node);
                stmts = quote! {
                    #stmts
                    let __child = {
                        #child_code
                        __el
                    };
                    ::typhoon_core::append_child(&__el, &__child);
                };
            }
            TpChild::Text(lit) => {
                stmts = quote! {
                    #stmts
                    ::typhoon_core::append_text_node(&__el, #lit);
                };
            }
            TpChild::Embed(expr) => {
                stmts = quote! {
                    #stmts
                    {
                        let __embedded = #expr;
                        ::typhoon_core::append_child(&__el, &__embedded);
                    }
                };
            }
        }
    }

    stmts
}

/// Write HTML-like element trees in Rust.
///
/// ```ignore
/// tp! {
///     div.class("app") {
///         h1.text("Hello")
///         button.onclick(my_handler) { "Click" }
///         (my_component())
///     }
/// }
/// ```
#[proc_macro]
pub fn tp(input: TokenStream) -> TokenStream {
    let TpInput(root) = parse_macro_input!(input as TpInput);
    let node_code = generate_node(&root);

    let expanded = quote! {
        {
            #node_code
            __el
        }
    };

    expanded.into()
}
