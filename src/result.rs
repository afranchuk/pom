use std::error;
use std::fmt::{self, Display};

#[derive(Clone)]
pub struct ErrorMessage<'a>(std::sync::Arc<dyn Fn() -> String + 'a>);

impl<'a> ErrorMessage<'a> {
	pub fn new<F: Fn() -> String + 'a>(f: F) -> Self {
		ErrorMessage(std::sync::Arc::new(f))
	}

	pub fn evaluate(self) -> String {
		(self.0)()
	}
}

impl<'a> fmt::Debug for ErrorMessage<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", (self.0)())
	}
}

/// Parser error.
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorM<Msg> {
	Incomplete,
	Mismatch {
		message: Msg,
		position: usize,
	},
	Conversion {
		message: Msg,
		position: usize,
	},
	Expect {
		message: Msg,
		position: usize,
		inner: Box<ErrorM<Msg>>,
	},
	Custom {
		message: Msg,
		position: usize,
		inner: Option<Box<ErrorM<Msg>>>,
	},
}

pub type Error = ErrorM<String>;
pub type ErrorDelayed<'a> = ErrorM<ErrorMessage<'a>>;

impl<'a> ErrorDelayed<'a> {
	pub fn evaluate(self) -> Error {
		match self {
			ErrorDelayed::Incomplete => Error::Incomplete,
			ErrorDelayed::Mismatch { message, position } => Error::Mismatch {
				message: message.evaluate(),
				position,
			},
			ErrorDelayed::Conversion { message, position } => Error::Conversion {
				message: message.evaluate(),
				position,
			},
			ErrorDelayed::Expect {
				message,
				position,
				inner,
			} => Error::Expect {
				message: message.evaluate(),
				position,
				inner: Box::new(inner.evaluate()),
			},
			ErrorDelayed::Custom {
				message,
				position,
				inner,
			} => Error::Custom {
				message: message.evaluate(),
				position,
				inner: inner.map(|inner| Box::new(inner.evaluate())),
			},
		}
	}
}

impl<Msg: Display + fmt::Debug> error::Error for ErrorM<Msg> {
	fn description(&self) -> &'static str {
		"Parse error"
	}
}

impl<Msg: Display> Display for ErrorM<Msg> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ErrorM::Incomplete => write!(f, "Incomplete"),
			ErrorM::Mismatch {
				ref message,
				ref position,
			} => write!(f, "Mismatch at {}: {}", position, message),
			ErrorM::Conversion {
				ref message,
				ref position,
			} => write!(f, "Conversion failed at {}: {}", position, message),
			ErrorM::Expect {
				ref message,
				ref position,
				ref inner,
			} => write!(f, "{} at {}: {}", message, position, inner),
			ErrorM::Custom {
				ref message,
				ref position,
				inner: Some(ref inner),
			} => write!(f, "{} at {}, (inner: {})", message, position, inner),
			ErrorM::Custom {
				ref message,
				ref position,
				inner: None,
			} => write!(f, "{} at {}", message, position),
		}
	}
}

/// Parser result, `Result<O>` ia alias of `Result<O, pom::Error>`.
pub type ResultDelayed<'a, O> = ::std::result::Result<O, ErrorDelayed<'a>>;
pub type Result<O> = std::result::Result<O, Error>;
