use crate::ast::prelude::*;

use ast::token::NumberSize;
use num::Num;

#[test]
#[cfg(not(miri))]
fn ast_parse() {
    rt::<ast::LitNumber>("42");
    rt::<ast::LitNumber>("42.42");
    rt::<ast::LitNumber>("0.42");
    rt::<ast::LitNumber>("0.42e10");
}

/// A number literal.
///
/// * `42`.
/// * `4.2e10`.
#[derive(Debug, TryClone, Clone, Copy, PartialEq, Eq, Spanned)]
#[try_clone(copy)]
#[non_exhaustive]
pub struct LitNumber {
    /// The span corresponding to the literal.
    pub span: Span,
    /// The source of the number.
    #[rune(skip)]
    pub source: ast::NumberSource,
}

impl ToAst for LitNumber {
    fn to_ast(span: Span, kind: ast::Kind) -> compile::Result<Self> {
        match kind {
            K![number(source)] => Ok(LitNumber { source, span }),
            _ => Err(compile::Error::expected(
                ast::Token { span, kind },
                Self::into_expectation(),
            )),
        }
    }

    #[inline]
    fn matches(kind: &ast::Kind) -> bool {
        matches!(kind, K![number])
    }

    #[inline]
    fn into_expectation() -> Expectation {
        Expectation::Description("number")
    }
}

impl Parse for LitNumber {
    fn parse(parser: &mut Parser<'_>) -> Result<Self> {
        let t = parser.next()?;
        Self::to_ast(t.span, t.kind)
    }
}

impl<'a> Resolve<'a> for LitNumber {
    type Output = ast::Number;

    fn resolve(&self, cx: ResolveContext<'a>) -> Result<ast::Number> {
        fn err_span<E>(span: Span) -> impl Fn(E) -> compile::Error {
            move |_| compile::Error::new(span, ErrorKind::BadNumberLiteral)
        }

        let span = self.span;

        let text = match self.source {
            ast::NumberSource::Synthetic(id) => {
                let Some(number) = cx.storage.get_number(id) else {
                    return Err(compile::Error::new(
                        span,
                        ErrorKind::BadSyntheticId {
                            kind: SyntheticKind::Number,
                            id,
                        },
                    ));
                };

                return Ok((*number).try_clone()?);
            }
            ast::NumberSource::Text(text) => text,
        };

        let string = cx
            .sources
            .source(text.source_id, text.number)
            .ok_or_else(|| compile::Error::new(span, ErrorKind::BadSlice))?;

        let suffix = cx
            .sources
            .source(text.source_id, text.suffix)
            .ok_or_else(|| compile::Error::new(span, ErrorKind::BadSlice))?;

        let suffix = match suffix {
            "u8" => Some(ast::NumberSuffix::Unsigned(text.suffix, NumberSize::S8)),
            "u16" => Some(ast::NumberSuffix::Unsigned(text.suffix, NumberSize::S16)),
            "u32" => Some(ast::NumberSuffix::Unsigned(text.suffix, NumberSize::S32)),
            "u64" => Some(ast::NumberSuffix::Unsigned(text.suffix, NumberSize::S64)),
            "i8" => Some(ast::NumberSuffix::Signed(text.suffix, NumberSize::S8)),
            "i16" => Some(ast::NumberSuffix::Signed(text.suffix, NumberSize::S16)),
            "i32" => Some(ast::NumberSuffix::Signed(text.suffix, NumberSize::S32)),
            "i64" => Some(ast::NumberSuffix::Signed(text.suffix, NumberSize::S64)),
            "f32" | "f64" => Some(ast::NumberSuffix::Float(text.suffix)),
            "" => None,
            _ => {
                return Err(compile::Error::new(
                    text.suffix,
                    ErrorKind::UnsupportedSuffix,
                ))
            }
        };

        if matches!(
            (suffix, text.is_fractional),
            (Some(ast::NumberSuffix::Float(..)), _) | (None, true)
        ) {
            let number: f64 = string
                .trim_matches(|c: char| c == '_')
                .parse()
                .map_err(err_span(span))?;

            return Ok(ast::Number {
                value: ast::NumberValue::Float(number),
                suffix,
            });
        }

        let radix = match text.base {
            ast::NumberBase::Binary => 2,
            ast::NumberBase::Octal => 8,
            ast::NumberBase::Hex => 16,
            ast::NumberBase::Decimal => 10,
        };

        let number = num::BigInt::from_str_radix(string, radix).map_err(err_span(span))?;

        Ok(ast::Number {
            value: ast::NumberValue::Integer(number),
            suffix,
        })
    }
}

impl ToTokens for LitNumber {
    fn to_tokens(
        &self,
        _: &mut MacroContext<'_, '_, '_>,
        stream: &mut TokenStream,
    ) -> alloc::Result<()> {
        stream.push(ast::Token {
            span: self.span,
            kind: ast::Kind::Number(self.source),
        })
    }
}
