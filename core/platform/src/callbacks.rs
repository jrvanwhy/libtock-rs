pub trait FreeCallback<AsyncResponse> {
    fn call(response: AsyncResponse);
}

pub trait MethodCallback<AsyncResponse> {
    fn call(&self, response: AsyncResponse);
}
