use std::error;
use std::fmt::{self, Display};

#[derive(Clone)]
pub struct ErrorMessage<'a>(std::sync::Arc<dyn Fn() -> String + 'a>);

impl<'a> ErrorMessage<'a> {
	pub fn new<F: Fn() -> String + 'a>(f: F) -> Self {
		ErrorMessage(std::sync::Arc::new(f))
	}

	pub fn evaluate(self) -> ErrorMessage<'static> {
		let s = self.to_string();
		ErrorMessage::new(move || s.clone())
	}
}

impl<'a> fmt::Debug for ErrorMessage<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", (self.0)())
	}
}

impl<'a> Display for ErrorMessage<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", (self.0)())
	}
}

/// Parser error.
#[derive(Debug, Clone)]
pub enum Error<'a> {
	Incomplete,
	Mismatch {
		message: ErrorMessage<'a>,
		position: usize,
	},
	Conversion {
		message: ErrorMessage<'a>,
		position: usize,
	},
	Expect {
		message: ErrorMessage<'a>,
		position: usize,
		inner: Box<Error<'a>>,
	},
	Custom {
		message: ErrorMessage<'a>,
		position: usize,
		inner: Option<Box<Error<'a>>>,
	},
}

impl<'a> Error<'a> {
	pub fn evaluate(self) -> Error<'static> {
		match self {
			Error::Incomplete => Error::Incomplete,
			Error::Mismatch { message, position } => Error::Mismatch {
				message: message.evaluate(),
				position,
			},
			Error::Conversion { message, position } => Error::Conversion {
				message: message.evaluate(),
				position,
			},
			Error::Expect {
				message,
				position,
				inner,
			} => Error::Expect {
				message: message.evaluate(),
				position,
				inner: Box::new(inner.evaluate()),
			},
			Error::Custom {
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

impl<'a, 'b> PartialEq<Error<'b>> for Error<'a> {
	fn eq(&self, other: &Error<'b>) -> bool {
		match (self, other) {
			(Error::Incomplete, Error::Incomplete) => true,
			(Error::Mismatch { .. }, Error::Mismatch { .. }) => true,
			(Error::Conversion { .. }, Error::Conversion { .. }) => true,
			(Error::Expect { .. }, Error::Expect { .. }) => true,
			(Error::Custom { .. }, Error::Custom { .. }) => true,
			_ => false,
		}
	}
}

impl<'a> error::Error for Error<'a> {
	fn description(&self) -> &'static str {
		"Parse error"
	}
}

impl<'a> Display for Error<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::Incomplete => write!(f, "Incomplete"),
			Error::Mismatch {
				ref message,
				ref position,
			} => write!(f, "Mismatch at {}: {}", position, message),
			Error::Conversion {
				ref message,
				ref position,
			} => write!(f, "Conversion failed at {}: {}", position, message),
			Error::Expect {
				ref message,
				ref position,
				ref inner,
			} => write!(f, "{} at {}: {}", message, position, inner),
			Error::Custom {
				ref message,
				ref position,
				inner: Some(ref inner),
			} => write!(f, "{} at {}, (inner: {})", message, position, inner),
			Error::Custom {
				ref message,
				ref position,
				inner: None,
			} => write!(f, "{} at {}", message, position),
		}
	}
}

/// Parser result, `Result<O>` ia alias of `Result<O, pom::Error>`.
pub type Result<'a, O> = ::std::result::Result<O, Error<'a>>;
