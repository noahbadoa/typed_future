use crate::core;

const fn empty_waker() -> core::task::Context::<'static>{
    let raw = core::task::Waker::noop();
    core::task::Context::from_waker(raw)
}

pub fn poll_empty<Output, T : core::future::Future<Output = Output>>(future : core::pin::Pin<&mut T>) -> core::task::Poll<Output>{
    future.poll(&mut empty_waker())
}

#[derive(Debug, Default)]
pub struct YeildOnceLocal(bool);
impl core::future::Future for YeildOnceLocal{
    type Output = ();
    fn poll(mut self: core::pin::Pin<&mut Self>, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if self.0{
            core::task::Poll::Ready(())
        }else{
            self.0 = true;
            core::task::Poll::Pending
        }
    }
}