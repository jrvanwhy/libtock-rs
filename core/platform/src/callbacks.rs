pub trait FreeCallback<AsyncResponse> {
    fn call(response: AsyncResponse);
}

pub trait MethodCallback<AsyncResponse>: Copy {
    fn call(self, response: AsyncResponse);
}
