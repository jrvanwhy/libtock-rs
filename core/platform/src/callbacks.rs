#[derive(Clone,Copy)]
pub struct CallbackContext<'c> {
    pub(crate) _private: (),
}

pub trait FreeCallback<AsyncResponse> {
    fn call(context: CallbackContext, response: AsyncResponse);
}

pub trait MethodCallback<AsyncResponse> {
    fn call(&self, response: AsyncResponse);
}
