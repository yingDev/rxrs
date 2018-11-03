use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;
use std::error::Error;

#[derive(Debug)]
pub struct RxError
{
    err: Arc<Error>,
    handled: bool,
}


impl Display for RxError
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result
    { write!(f, "RxError: {:}; handled={}", self.err, self.handled) }
}

impl Clone for RxError
{
    fn clone(&self) -> Self { RxError{ err: self.err.clone(), handled: false } }
}

impl RxError
{
    pub fn new(e: impl Error+'static) -> Self
    {
        RxError{ err: Arc::new(e), handled: false }
    }

    pub fn set_handled(self) -> Self
    {
        let mut s = self;
        s.handled = true;
        s
    }

    pub fn simple(source: Option<Arc<Error+'static>>, msg: impl Into<String>) -> Self
    {
        #[derive(Debug)]
        struct StrError(Option<Arc<Error+'static>>, String);
        impl Display for StrError
        {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result
            { write!(f, "RxError: {}; source={:?}", self.1, self.0) }
        }
        impl Error for StrError
        {
            fn source(&self) -> Option<&(dyn Error+'static)> {
                match &self.0 {
                    Some(arc) => Some(Arc::as_ref(arc)),
                    None => None
                }
            }
        }

        Self::new(StrError(source, msg.into()))

    }

    pub fn handle(mut self, f: impl Fn(&Error) -> Option<RxError>) -> Option<RxError>
    {
        let out = f(self.err.as_ref());
        self.handled = true;
        out
    }
}

impl Drop for RxError
{
    fn drop(&mut self)
    {
        if ! self.handled {
            panic!("Unhandled Error: {}", self.err )
        }
    }
}