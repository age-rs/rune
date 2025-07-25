use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;

#[derive(Default)]
enum Path {
    #[default]
    None,
    Rename(syn::PathSegment),
    Protocol(syn::Path),
}

#[derive(Default)]
pub(crate) struct FunctionAttrs {
    instance: bool,
    /// A free function.
    free: bool,
    /// Keep the existing function in place, and generate a separate hidden meta function.
    keep: bool,
    /// Path to register in.
    path: Path,
    /// Looks like an associated type.
    self_type: Option<syn::PathSegment>,
    /// Defines a fallible function which can make use of the `?` operator.
    vm_result: bool,
    /// The function is deprecated.
    deprecated: Option<syn::LitStr>,
}

impl FunctionAttrs {
    /// Parse the given parse stream.
    pub(crate) fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = Self::default();

        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;

            if ident == "instance" {
                out.instance = true;
            } else if ident == "free" {
                out.free = true;
            } else if ident == "keep" {
                out.keep = true;
            } else if ident == "vm_result" {
                out.vm_result = true;
            } else if ident == "protocol" {
                input.parse::<Token![=]>()?;
                let protocol: syn::Path = input.parse()?;
                out.path = Path::Protocol(if let Some(protocol) = protocol.get_ident() {
                    syn::Path {
                        leading_colon: None,
                        segments: ["rune", "runtime", "Protocol"]
                            .into_iter()
                            .map(|i| syn::Ident::new(i, protocol.span()))
                            .chain(Some(protocol.clone()))
                            .map(syn::PathSegment::from)
                            .collect(),
                    }
                } else {
                    protocol
                })
            } else if ident == "path" {
                input.parse::<Token![=]>()?;

                let path = input.parse::<syn::Path>()?;

                if path.segments.len() > 2 {
                    return Err(syn::Error::new_spanned(
                        path,
                        "Expected at most two path segments",
                    ));
                }

                let mut it = path.segments.into_iter();

                let Some(first) = it.next() else {
                    return Err(syn::Error::new(
                        input.span(),
                        "Expected at least one path segment",
                    ));
                };

                if let Some(second) = it.next() {
                    let syn::PathArguments::None = &first.arguments else {
                        return Err(syn::Error::new_spanned(
                            first.arguments,
                            "Unsupported arguments",
                        ));
                    };

                    out.self_type = Some(first);
                    out.path = Path::Rename(second);
                } else if first.ident == "Self" {
                    out.self_type = Some(first);
                } else {
                    out.path = Path::Rename(first);
                }
            } else if ident == "deprecated" {
                input.parse::<Token![=]>()?;
                out.deprecated = Some(input.parse()?);
            } else {
                return Err(syn::Error::new_spanned(ident, "Unsupported option"));
            }

            if input.parse::<Option<Token![,]>>()?.is_none() {
                break;
            }
        }

        let stream = input.parse::<TokenStream>()?;

        if !stream.is_empty() {
            return Err(syn::Error::new_spanned(stream, "Unexpected input"));
        }

        Ok(out)
    }
}

pub(crate) struct Function {
    attributes: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: syn::Signature,
    remainder: TokenStream,
    docs: syn::ExprArray,
    arguments: syn::ExprArray,
    takes_self: bool,
}

impl Function {
    /// Parse the given parse stream.
    pub(crate) fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed_attributes = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<syn::Visibility>()?;
        let sig = input.parse::<syn::Signature>()?;

        let mut attributes = Vec::new();

        let mut docs = syn::ExprArray {
            attrs: Vec::new(),
            bracket_token: syn::token::Bracket::default(),
            elems: Punctuated::default(),
        };

        for attr in parsed_attributes {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(name_value) = &attr.meta {
                    docs.elems.push(name_value.value.clone());
                }
            }

            attributes.push(attr);
        }

        let mut arguments = syn::ExprArray {
            attrs: Vec::new(),
            bracket_token: syn::token::Bracket::default(),
            elems: Punctuated::default(),
        };

        let mut takes_self = false;

        for arg in &sig.inputs {
            let argument_name = match arg {
                syn::FnArg::Typed(ty) => argument_ident(&ty.pat),
                syn::FnArg::Receiver(..) => {
                    takes_self = true;
                    syn::LitStr::new("self", arg.span())
                }
            };

            arguments.elems.push(syn::Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::Lit::Str(argument_name),
            }));
        }

        let remainder = input.parse::<TokenStream>()?;

        Ok(Self {
            attributes,
            vis,
            sig,
            remainder,
            docs,
            arguments,
            takes_self,
        })
    }

    /// Expand the function declaration.
    pub(crate) fn expand(mut self, attrs: FunctionAttrs) -> syn::Result<TokenStream> {
        let instance = attrs.instance || self.takes_self;

        let (meta_fn, real_fn, mut sig, real_fn_mangled) = if attrs.keep {
            let meta_fn =
                syn::Ident::new(&format!("{}__meta", self.sig.ident), self.sig.ident.span());
            let real_fn = self.sig.ident.clone();
            (meta_fn, real_fn, self.sig.clone(), false)
        } else {
            let meta_fn = self.sig.ident.clone();
            let real_fn = syn::Ident::new(
                &format!("__rune_fn__{}", self.sig.ident),
                self.sig.ident.span(),
            );
            let mut sig = self.sig.clone();
            sig.ident = real_fn.clone();
            (meta_fn, real_fn, sig, true)
        };

        let mut path = syn::Path {
            leading_colon: None,
            segments: Punctuated::default(),
        };

        match (self.takes_self, attrs.free, &attrs.self_type) {
            (true, _, _) => {
                path.segments
                    .push(syn::PathSegment::from(<Token![Self]>::default()));
                path.segments.push(syn::PathSegment::from(real_fn));
            }
            (_, false, Some(self_type)) => {
                path.segments.push(self_type.clone());
                path.segments.push(syn::PathSegment::from(real_fn));
            }
            _ => {
                path.segments.push(syn::PathSegment::from(real_fn));
            }
        }

        let real_fn_path = path;

        let name_string = syn::LitStr::new(&self.sig.ident.to_string(), self.sig.ident.span());

        let name = if instance {
            'out: {
                syn::Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Str(match &attrs.path {
                        Path::Protocol(protocol) => {
                            break 'out syn::parse_quote!(&#protocol);
                        }
                        Path::None => name_string.clone(),
                        Path::Rename(last) => {
                            syn::LitStr::new(&last.ident.to_string(), last.ident.span())
                        }
                    }),
                })
            }
        } else {
            match &attrs.path {
                Path::None => expr_lit(&self.sig.ident),
                Path::Rename(last) => expr_lit(&last.ident),
                Path::Protocol(protocol) => syn::parse_quote!(&#protocol),
            }
        };

        let arguments = match &attrs.path {
            Path::None | Path::Protocol(_) => Punctuated::default(),
            Path::Rename(last) => match &last.arguments {
                syn::PathArguments::AngleBracketed(arguments) => arguments.args.clone(),
                syn::PathArguments::None => Punctuated::default(),
                arguments => {
                    return Err(syn::Error::new_spanned(
                        arguments,
                        "Unsupported path segments",
                    ));
                }
            },
        };

        let name = if !arguments.is_empty() {
            let mut array = syn::ExprArray {
                attrs: Vec::new(),
                bracket_token: <syn::token::Bracket>::default(),
                elems: Punctuated::default(),
            };

            for argument in arguments {
                array.elems.push(syn::Expr::Verbatim(quote! {
                    <#argument as rune::__priv::TypeHash>::HASH
                }));
            }

            quote!(rune::__priv::Params::new(#name, #array))
        } else {
            quote!(#name)
        };

        if instance {
            // Ensure that the first argument is called `self`.
            if let Some(argument) = self.arguments.elems.first_mut() {
                let span = argument.span();

                *argument = syn::Expr::Lit(syn::ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Str(syn::LitStr::new("self", span)),
                });
            }
        }

        let meta_kind = syn::Ident::new(
            if instance { "instance" } else { "function" },
            self.sig.span(),
        );

        let mut stream = TokenStream::new();

        for attr in self.attributes {
            stream.extend(attr.into_token_stream());
        }

        if real_fn_mangled {
            stream.extend(quote!(#[allow(non_snake_case)]));
            stream.extend(quote!(#[doc(hidden)]));
        }

        stream.extend(self.vis.to_token_stream());

        let vm_result = VmResult::new();

        if attrs.vm_result {
            let VmResult {
                result, vm_error, ..
            } = &vm_result;

            sig.output = match sig.output {
                syn::ReturnType::Default => syn::ReturnType::Type(
                    <Token![->]>::default(),
                    Box::new(syn::Type::Verbatim(quote!(#result<(), #vm_error>))),
                ),
                syn::ReturnType::Type(arrow, ty) => syn::ReturnType::Type(
                    arrow,
                    Box::new(syn::Type::Verbatim(quote!(#result<#ty, #vm_error>))),
                ),
            };
        }

        let generics = sig.generics.clone();
        stream.extend(sig.into_token_stream());

        if attrs.vm_result {
            let mut block: syn::Block = syn::parse2(self.remainder)?;
            vm_result.block(&mut block, true)?;
            block.to_tokens(&mut stream);
        } else {
            stream.extend(self.remainder);
        }

        let arguments = &self.arguments;
        let docs = &self.docs;

        let build_with = if instance {
            None
        } else if let Some(self_type) = &attrs.self_type {
            Some(quote!(.build_associated::<#self_type>()?))
        } else {
            Some(quote!(.build()?))
        };

        let attributes = (!real_fn_mangled).then(|| quote!(#[allow(non_snake_case)]));

        let deprecated = match &attrs.deprecated {
            Some(message) => quote!(Some(#message)),
            None => quote!(None),
        };

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let type_generics = type_generics.as_turbofish();

        stream.extend(quote! {
            /// Get function metadata.
            #[automatically_derived]
            #attributes
            #[doc(hidden)]
            pub(crate) fn #meta_fn #impl_generics() -> Result<rune::__priv::FunctionMetaData, rune::alloc::Error>
            #where_clause
            {
                Ok(rune::__priv::FunctionMetaData {
                    kind: rune::__priv::FunctionMetaKind::#meta_kind(#name, #real_fn_path #type_generics)?#build_with,
                    statics: rune::__priv::FunctionMetaStatics {
                        name: #name_string,
                        deprecated: #deprecated,
                        docs: &#docs[..],
                        arguments: &#arguments[..],
                    },
                })
            }
        });

        Ok(stream)
    }
}

/// The identifier of an argument.
fn argument_ident(pat: &syn::Pat) -> syn::LitStr {
    match pat {
        syn::Pat::Type(pat) => argument_ident(&pat.pat),
        syn::Pat::Path(pat) => argument_path_ident(&pat.path),
        syn::Pat::Ident(pat) => syn::LitStr::new(&pat.ident.to_string(), pat.span()),
        _ => syn::LitStr::new(&pat.to_token_stream().to_string(), pat.span()),
    }
}

/// Argument path identifier.
fn argument_path_ident(path: &syn::Path) -> syn::LitStr {
    match path.get_ident() {
        Some(ident) => syn::LitStr::new(&ident.to_string(), path.span()),
        None => syn::LitStr::new(&path.to_token_stream().to_string(), path.span()),
    }
}

fn expr_lit(ident: &syn::Ident) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit: syn::Lit::Str(syn::LitStr::new(&ident.to_string(), ident.span())),
    })
}

struct VmResult {
    result: syn::Path,
    from: syn::Path,
    vm_error: syn::Path,
}

impl VmResult {
    fn new() -> Self {
        Self {
            result: syn::parse_quote!(::core::result::Result),
            from: syn::parse_quote!(::core::convert::From),
            vm_error: syn::parse_quote!(rune::VmError),
        }
    }

    /// Modify the block so that it is fallible.
    fn block(&self, ast: &mut syn::Block, top_level: bool) -> syn::Result<()> {
        let result = &self.result;

        for stmt in &mut ast.stmts {
            match stmt {
                syn::Stmt::Expr(expr, _) => {
                    self.expr(expr)?;
                }
                syn::Stmt::Local(local) => {
                    let Some(init) = &mut local.init else {
                        continue;
                    };

                    self.expr(&mut init.expr)?;

                    let Some((_, expr)) = &mut init.diverge else {
                        continue;
                    };

                    self.expr(expr)?;
                }
                _ => {}
            };
        }

        if top_level {
            let mut found = false;

            for stmt in ast.stmts.iter_mut().rev() {
                if let syn::Stmt::Expr(expr, semi) = stmt {
                    if semi.is_none() {
                        found = true;

                        *expr = syn::Expr::Verbatim(quote_spanned! {
                            expr.span() => #result::Ok(#expr)
                        });
                    }

                    break;
                }
            }

            if !found {
                ast.stmts.push(syn::Stmt::Expr(
                    syn::Expr::Verbatim(quote!(#result::Ok(()))),
                    None,
                ));
            }
        }

        Ok(())
    }

    fn expr(&self, ast: &mut syn::Expr) -> syn::Result<()> {
        let Self { result, from, .. } = self;

        let outcome = 'outcome: {
            match ast {
                syn::Expr::Array(expr) => {
                    for expr in &mut expr.elems {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Assign(expt) => {
                    self.expr(&mut expt.right)?;
                }
                syn::Expr::Async(..) => {}
                syn::Expr::Await(expr) => {
                    self.expr(&mut expr.base)?;
                }
                syn::Expr::Binary(expr) => {
                    self.expr(&mut expr.left)?;
                    self.expr(&mut expr.right)?;
                }
                syn::Expr::Block(block) => {
                    self.block(&mut block.block, false)?;
                }
                syn::Expr::Break(expr) => {
                    if let Some(expr) = &mut expr.expr {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Call(expr) => {
                    self.expr(&mut expr.func)?;

                    for expr in &mut expr.args {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Field(expr) => {
                    self.expr(&mut expr.base)?;
                }
                syn::Expr::ForLoop(expr) => {
                    self.expr(&mut expr.expr)?;
                    self.block(&mut expr.body, false)?;
                }
                syn::Expr::Group(expr) => {
                    self.expr(&mut expr.expr)?;
                }
                syn::Expr::If(expr) => {
                    self.expr(&mut expr.cond)?;
                    self.block(&mut expr.then_branch, false)?;

                    if let Some((_, expr)) = &mut expr.else_branch {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Index(expr) => {
                    self.expr(&mut expr.expr)?;
                    self.expr(&mut expr.index)?;
                }
                syn::Expr::Let(expr) => {
                    self.expr(&mut expr.expr)?;
                }
                syn::Expr::Loop(expr) => {
                    self.block(&mut expr.body, false)?;
                }
                syn::Expr::Match(expr) => {
                    self.expr(&mut expr.expr)?;

                    for arm in &mut expr.arms {
                        if let Some((_, expr)) = &mut arm.guard {
                            self.expr(expr)?;
                        }

                        self.expr(&mut arm.body)?;
                    }
                }
                syn::Expr::MethodCall(expr) => {
                    self.expr(&mut expr.receiver)?;

                    for expr in &mut expr.args {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Paren(expr) => {
                    self.expr(&mut expr.expr)?;
                }
                syn::Expr::Range(expr) => {
                    if let Some(expr) = &mut expr.start {
                        self.expr(expr)?;
                    }

                    if let Some(expr) = &mut expr.end {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Reference(expr) => {
                    self.expr(&mut expr.expr)?;
                }
                syn::Expr::Repeat(expr) => {
                    self.expr(&mut expr.expr)?;
                    self.expr(&mut expr.len)?;
                }
                syn::Expr::Return(expr) => {
                    if let Some(expr) = &mut expr.expr {
                        self.expr(expr)?;
                    }

                    expr.expr = Some(Box::new(match expr.expr.take() {
                        Some(expr) => syn::Expr::Verbatim(quote_spanned! {
                            expr.span() =>
                            #result::Ok(#expr)
                        }),
                        None => syn::Expr::Verbatim(quote!(#result::Ok(()))),
                    }));
                }
                syn::Expr::Struct(expr) => {
                    for field in &mut expr.fields {
                        self.expr(&mut field.expr)?;
                    }
                }
                syn::Expr::Try(expr) => {
                    let span = expr.span();

                    self.expr(&mut expr.expr)?;

                    break 'outcome if let Some(expr) = as_vm_expr(&mut expr.expr) {
                        quote_spanned!(span => #expr?)
                    } else {
                        let value = &mut expr.expr;

                        quote_spanned! {
                            span =>
                            match #value {
                                #result::Ok(value) => value,
                                #result::Err(error) => {
                                    return #result::Ok(#result::Err(#[allow(clippy::useless_conversion)] #from::from(error)));
                                }
                            }
                        }
                    };
                }
                syn::Expr::Tuple(expr) => {
                    for expr in &mut expr.elems {
                        self.expr(expr)?;
                    }
                }
                syn::Expr::Unary(expr) => {
                    self.expr(&mut expr.expr)?;
                }
                syn::Expr::Unsafe(expr) => {
                    self.block(&mut expr.block, false)?;
                }
                syn::Expr::While(expr) => {
                    self.expr(&mut expr.cond)?;
                    self.block(&mut expr.body, false)?;
                }
                syn::Expr::Yield(expr) => {
                    if let Some(expr) = &mut expr.expr {
                        self.expr(expr)?;
                    }
                }
                _ => {}
            }

            return Ok(());
        };

        *ast = syn::Expr::Verbatim(outcome);
        Ok(())
    }
}

/// If this is a field expression like `<expr>.vm`.
fn as_vm_expr(expr: &mut syn::Expr) -> Option<&mut syn::Expr> {
    let syn::Expr::Field(expr) = expr else {
        return None;
    };

    let syn::Member::Named(ident) = &expr.member else {
        return None;
    };

    (ident == "vm").then_some(&mut expr.base)
}
