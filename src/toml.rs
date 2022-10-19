//! TOML specific features. This module only exists if the Cargo feature `toml`
//! is enabled.

use std::fmt::{self, Write};

use crate::{
    Config,
    format::{self, Formatter},
    meta::Expr,
};



/// Options for generating a TOML template.
pub struct FormatOptions {
    /// Indentation for nested tables. Default: 0.
    pub indent: u8,

    /// Non-TOML specific options.
    general: format::Options,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: 0,
            general: Default::default(),
        }
    }
}

/// Formats the configuration description as a TOML file.
///
/// This can be used to generate a template file that you can give to the users
/// of your application. It usually is a convenient to start with a correctly
/// formatted file with all possible options inside.
///
/// # Example
///
/// ```
/// # use pretty_assertions::assert_eq;
/// use std::path::PathBuf;
/// use confique::{Config, toml::FormatOptions};
///
/// /// App configuration.
/// #[derive(Config)]
/// struct Conf {
///     /// The color of the app.
///     color: String,
///
///     #[config(nested)]
///     log: LogConfig,
/// }
///
/// #[derive(Config)]
/// struct LogConfig {
///     /// If set to `true`, the app will log to stdout.
///     #[config(default = true)]
///     stdout: bool,
///
///     /// If this is set, the app will write logs to the given file. Of course,
///     /// the app has to have write access to that file.
///     #[config(env = "LOG_FILE")]
///     file: Option<PathBuf>,
/// }
///
/// const EXPECTED: &str = "\
/// ## App configuration.
///
/// ## The color of the app.
/// ##
/// ## Required! This value must be specified.
/// ##color =
///
/// [log]
/// ## If set to `true`, the app will log to stdout.
/// ##
/// ## Default value: true
/// ##stdout = true
///
/// ## If this is set, the app will write logs to the given file. Of course,
/// ## the app has to have write access to that file.
/// ##
/// ## Can also be specified via environment variable `LOG_FILE`.
/// ##file =
/// ";
///
/// fn main() {
///     let toml = confique::toml::format::<Conf>(FormatOptions::default());
///     assert_eq!(toml, EXPECTED);
/// }
/// ```
pub fn format<C: Config>(options: FormatOptions) -> String {
    let mut out = TomlFormatter::new(&options);
    format::format::<C>(&mut out, options.general);
    out.finish()
}

struct TomlFormatter {
    indent: u8,
    buffer: String,
    stack: Vec<&'static str>,
}

impl TomlFormatter {
    fn new(options: &FormatOptions) -> Self {
        Self {
            indent: options.indent,
            buffer: String::new(),
            stack: Vec::new(),
        }
    }

    fn emit_indentation(&mut self) {
        let num_spaces = self.stack.len() * self.indent as usize;
        write!(self.buffer, "{: <1$}", "", num_spaces).unwrap();
    }
}

impl Formatter for TomlFormatter {
    type ExprPrinter = PrintExpr;

    fn buffer(&mut self) -> &mut String {
        &mut self.buffer
    }

    fn comment(&mut self, comment: impl fmt::Display) {
        self.emit_indentation();
        writeln!(self.buffer, "#{comment}").unwrap();
    }

    fn disabled_field(&mut self, name: &str, value: Option<&'static Expr>) {
        match value.map(PrintExpr) {
            None => self.comment(format_args!("{name} =")),
            Some(v) => self.comment(format_args!("{name} = {v}")),
        };
    }

    fn start_nested(&mut self, name: &'static str, doc: &[&'static str]) {
        self.stack.push(name);
        doc.iter().for_each(|doc| self.comment(doc));
        self.emit_indentation();
        writeln!(self.buffer, "[{}]", self.stack.join(".")).unwrap();
    }

    fn end_nested(&mut self) {
        self.stack.pop().expect("formatter bug: stack empty");
    }

    fn start_main(&mut self) {
        self.make_gap(1);
    }

    fn finish(self) -> String {
        assert!(self.stack.is_empty(), "formatter bug: stack not empty");
        self.buffer
    }
}

/// Helper to emit `meta::Expr` into TOML.
struct PrintExpr(&'static Expr);

impl From<&'static Expr> for PrintExpr {
    fn from(expr: &'static Expr) -> Self {
        Self(expr)
    }
}

impl fmt::Display for PrintExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        toml::to_string(&self.0)
            .expect("string serialization to TOML failed")
            .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{self, include_format_output};
    use super::{format, FormatOptions};
    use pretty_assertions::assert_str_eq;

    #[test]
    fn default() {
        let out = format::<test_utils::example1::Conf>(FormatOptions::default());
        assert_str_eq!(&out, include_format_output!("1-default.toml"));
    }

    #[test]
    fn no_comments() {
        let mut options = FormatOptions::default();
        options.general.comments = false;
        let out = format::<test_utils::example1::Conf>(options);
        assert_str_eq!(&out, include_format_output!("1-no-comments.toml"));
    }

    #[test]
    fn indent_2() {
        let mut options = FormatOptions::default();
        options.indent = 2;
        let out = format::<test_utils::example1::Conf>(options);
        assert_str_eq!(&out, include_format_output!("1-indent-2.toml"));
    }

    #[test]
    fn nested_gap_2() {
        let mut options = FormatOptions::default();
        options.general.nested_field_gap = 2;
        let out = format::<test_utils::example1::Conf>(options);
        assert_str_eq!(&out, include_format_output!("1-nested-gap-2.toml"));
    }

    #[test]
    fn immediately_nested() {
        let out = format::<test_utils::example2::Conf>(Default::default());
        assert_str_eq!(&out, include_format_output!("2-default.toml"));
    }
}
